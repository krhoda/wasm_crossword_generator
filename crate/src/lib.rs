mod utils;

use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    thread_rng, Rng,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::{Ord, Ordering},
    collections::HashSet,
};
use thiserror::Error;
use tsify::Tsify;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

/* Uncomment for Debug
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
*/

// If a word is smaller than three letters, it will make the crossword generation work less well.
// Defined as a constant to avoid magic numbers.
const MIN_LETTER_COUNT: usize = 3;

// Word is a record containing a potential portion of the Crossword answer at "text"
// along with an optional "clue" field.
// The "text" field will be split using .chars() with all the implications that brings.
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Word {
    pub text: String,
    pub clue: Option<String>,
}

// Direction is used to determine the orientation of the word.
#[derive(Clone, Debug, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Direction {
    Horizontal,
    Verticle,
}

// The implementation of Direction exposes a logical "not" fn called "other"
impl Direction {
    pub fn other(&self) -> Direction {
        match self {
            Direction::Horizontal => Direction::Verticle,
            Direction::Verticle => Direction::Horizontal,
        }
    }
}

// This impl block adds the ability to get a random Direction using the local rng
impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=1) {
            0 => Direction::Horizontal,
            _ => Direction::Verticle,
        }
    }
}

// PlacedWord represents a word which makes up the crossword puzzle's answers
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PlacedWord {
    // TODO: Add x / y coord of origin.
    pub direction: Direction,
    pub word: Word,
}

// CrosswordRow represents of one row of a crossword answer
// Within the interior "row" vector, there is either a None for a blank space
// or a Some(c) where c: char which would be a component of the crossword solution.
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordRow {
    pub row: Vec<Option<char>>,
}

// NOTE: If const generics worked with wasm_bindgen, we could do something way more concise/clear
impl CrosswordRow {
    fn new(width: usize) -> CrosswordRow {
        let mut row = Vec::<Option<char>>::new();
        for _ in 0..width {
            row.push(None);
        }

        CrosswordRow { row }
    }
}

// Placement describes a location on the crossword puzzle.
// TODO: Replace .direction in PlacedWord with .placement?
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Placement {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
}

// Crossword represents a complete crossword puzzle structure. Does not include stateful
// constructs for user input, just represents the static structure and answers.
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Crossword {
    pub puzzle: Vec<CrosswordRow>,
    pub words: Vec<PlacedWord>,
    width: usize,
    height: usize,
}

// CrosswordError is what it says on the tin. Often ellided over in favor of retrying, which if
// fails, throws CrosswordError::MaxRetries.
#[derive(Error, Debug)]
pub enum CrosswordError {
    #[error("word doesn't fit")]
    BadFit,
    #[error("intersection point was empty on non-first word")]
    EmptyIntersection,
    #[error("could not generate crossword before hitting max retries")]
    MaxRetries,
    #[error("point out of bounds")]
    PointOutOfBounds,
}

// CrosswordReqs is requirements the final puzzle must meet such as...
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordReqs {
    // ... how many times to attempt to make a valid puzzle before erroring out ...
    pub max_retries: usize, // NOTE: This is the only required one
    // ... how small can a word be ...
    pub min_letters_per_word: Option<usize>,
    // ... the minimum number of words that make up the answer ...
    pub min_words: Option<usize>,
    // ... the number of columns that can be completely empty ...
    pub max_empty_columns: Option<usize>,
    // ... the number of rows that can be completely empty
    pub max_empty_rows: Option<usize>,
}

// CrosswordInitialPlacementStrategy allows the caller to specifiy how to begin the crossword
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum CrosswordInitialPlacementStrategy {
    Center(Direction),
    Custom(Placement),
    UpperLeft,
    LowerLeft,
    UpperRight,
    LowerRight,
}

impl std::default::Default for CrosswordInitialPlacementStrategy {
    fn default() -> Self {
        CrosswordInitialPlacementStrategy::Center(Direction::Horizontal)
    }
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordInitialPlacement {
    pub min_letter_count: Option<usize>,
    pub strategy: Option<CrosswordInitialPlacementStrategy>,
}

impl std::default::Default for CrosswordInitialPlacement {
    fn default() -> Self {
        CrosswordInitialPlacement {
            min_letter_count: Some(MIN_LETTER_COUNT),
            strategy: Some(CrosswordInitialPlacementStrategy::default()),
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordConf {
    pub words: Vec<Word>,
    pub max_words: usize,
    pub width: usize,
    pub height: usize,
    pub requirements: Option<CrosswordReqs>,
    pub initial_placement: Option<CrosswordInitialPlacement>,
}

// This is the way calling applications should construct Crossword structs
#[wasm_bindgen]
pub fn new_crossword(conf: CrosswordConf) -> Result<Crossword, JsError> {
    // This call improves err handling on the JS side.
    // This fn should be the entry point from WASM so it makes sense to call this here.
    set_panic_hook();
    Crossword::new(conf).map_err(|e| JsError::new(&e.to_string()))
}

// NOTE: This impl has no pub fns, it's used by new_crossword
#[wasm_bindgen]
impl Crossword {
    fn new(conf: CrosswordConf) -> Result<Crossword, CrosswordError> {
        Crossword::new_recursive(conf, 0)
    }

    fn new_recursive(conf: CrosswordConf, iteration: usize) -> Result<Crossword, CrosswordError> {
        // To avoid issues with borrowing later.
        let c = conf.clone();

        let mut crossword = Crossword::new_empty(conf.width, conf.height);

        // Check the config for a min letter count, otherwise set to the constant.
        let min_letter_count = if let Some(reqs) = &conf.requirements {
            match reqs.min_letters_per_word {
                None => MIN_LETTER_COUNT,
                Some(mlc) => {
                    if mlc >= MIN_LETTER_COUNT {
                        mlc
                    } else {
                        MIN_LETTER_COUNT
                    }
                }
            }
        } else {
            MIN_LETTER_COUNT
        };

        let mut words = conf.words;
        // Remove all words under the min letter count
        words.retain(|w| w.text.chars().count() >= min_letter_count);
        // Remove all duplicate words
        // let mut h = HashSet::new();
        // words.retain(|w| h.insert(*w));

        let max_words = conf.max_words;

        words.shuffle(&mut thread_rng());

        for word in words {
            // If max words hit, break.
            if crossword.words.len() >= max_words {
                break;
            }

            // Detect if this is the initial placement.
            if crossword.words.is_empty() {
                // TODO: Consult the config for starting at a particular index!
                let _ = crossword.place(word, 0, 0, 0);
                continue;
            }

            for (intersection_index, letter) in word.text.chars().enumerate() {
                if Crossword::place_word(&mut crossword, intersection_index, &word, &letter)
                    .is_some()
                {
                    break;
                }
            }
        }

        match conf.requirements {
            Some(req) => {
                let mut valid = true;
                if let Some(min_words) = req.min_words {
                    if crossword.words.len() < min_words {
                        valid = false;
                    }
                }

                if valid {
                    if let Some(mer) = req.max_empty_rows {
                        if mer < crossword.empty_rows() {
                            valid = false;
                        }
                    }
                }

                if valid {
                    if let Some(mec) = req.max_empty_columns {
                        if mec < crossword.empty_columns() {
                            valid = false;
                        }
                    }
                }

                if valid {
                    Ok(crossword)
                } else if iteration < req.max_retries {
                    Crossword::new_recursive(c, iteration + 1)
                } else {
                    Err(CrosswordError::MaxRetries)
                }
            }
            None => Ok(crossword),
        }
    }

    // This will return Some(()) if the letter was placed.
    fn place_word(
        crossword: &mut Crossword,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        let temp = crossword.clone();
        for (row_count, _) in temp.puzzle.iter().enumerate() {
            if Crossword::place_on_row(crossword, row_count, intersection_index, word, letter)
                .is_some()
            {
                return Some(());
            }
        }

        None
    }

    // This will return Some(()) if the letter was placed.
    fn place_on_row(
        crossword: &mut Crossword,
        row_count: usize,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        let temp = crossword.clone();
        for (col_count, _) in temp.puzzle[row_count].row.iter().enumerate() {
            if Crossword::place_on_column(
                crossword,
                col_count,
                row_count,
                intersection_index,
                word,
                letter,
            )
            .is_some()
            {
                return Some(());
            }
        }

        None
    }

    // This will return Some(()) if the letter was placed.
    fn place_on_column(
        crossword: &mut Crossword,
        col_count: usize,
        row_count: usize,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        if let Some(c) = crossword.puzzle[row_count].row[col_count] {
            if &c == letter
                && crossword
                    .place(word.clone(), col_count, row_count, intersection_index)
                    .is_ok()
            {
                return Some(());
            }
        }

        None
    }

    // Returns an empty crossword at the given w x h
    fn new_empty(width: usize, height: usize) -> Crossword {
        let mut puzzle = Vec::<CrosswordRow>::new();

        for _ in 0..height {
            puzzle.push(CrosswordRow::new(width))
        }

        Crossword {
            puzzle,
            words: Vec::<PlacedWord>::new(),
            width,
            height,
        }
    }

    // A helper function to determine if any words have been placed to allow special casing of
    // the first word.
    fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    // returns a count of completely empty columns
    fn empty_columns(&self) -> usize {
        let mut acc = 0;
        for i in 0..self.width {
            let mut is_empty = true;
            for row in self.puzzle.clone() {
                if row.row[i].is_some() {
                    is_empty = false;
                    break;
                }
            }

            if is_empty {
                acc += 1;
            }
        }

        acc
    }

    // returns a count of completely empty rows
    fn empty_rows(&self) -> usize {
        let mut acc = 0;
        for row in self.puzzle.clone() {
            let mut is_empty = true;
            for space in row.row {
                if space.is_some() {
                    is_empty = false;
                    break;
                }
            }

            if is_empty {
                acc += 1;
            }
        }
        acc
    }

    // place takes a word and a possible intersection of the word.
    // The first word is special cased in the interior fn call to can_place.
    fn place(
        &mut self,
        word: Word,
        x: usize,
        y: usize,
        intersection_index: usize,
    ) -> Result<(), CrosswordError> {
        if x > (self.width - 1) || y > (self.height - 1) {
            return Err(CrosswordError::PointOutOfBounds);
        }

        // Can the word be placed at these coordinates? If so, which direction?
        // In the case that either direction is valid, Cthulu has risen, but also,
        // it would tolerare this non-Eucludian condition and pick randomly.
        let placement_direction = self.can_place(&word, x, y, intersection_index);

        if let Some(direction) = placement_direction {
            let origin = if let Direction::Horizontal = direction {
                x
            } else {
                y
            };

            for (letter_count, letter) in word.text.chars().enumerate() {
                // Set the next index relative to the intersection_index.
                let next_index = match letter_count.cmp(&intersection_index) {
                    Ordering::Less => origin - (intersection_index - letter_count),
                    Ordering::Equal => origin,
                    Ordering::Greater => origin + (letter_count - intersection_index),
                };

                match direction {
                    Direction::Horizontal => {
                        self.puzzle[y].row[next_index] = Some(letter);
                    }
                    Direction::Verticle => {
                        self.puzzle[next_index].row[x] = Some(letter);
                    }
                };
            }

            self.words.push(PlacedWord { word, direction });
            return Ok(());
        }

        Err(CrosswordError::BadFit)
    }

    fn can_place(
        &mut self,
        word: &Word,
        x: usize,
        y: usize,
        intersection_index: usize,
    ) -> Option<Direction> {
        // Sanity check:
        if intersection_index > word.text.len() - 1 {
            return None;
        }

        if !self.is_empty() {
            let intersection_letter_word =
                word.text.chars().collect::<Vec<char>>()[intersection_index];
            if let Some(intersection_letter_puzzle) = self.puzzle[y].row[x] {
                if intersection_letter_puzzle != intersection_letter_word {
                    return None;
                }
            };
        }

        let first_try: Direction = rand::random();
        let second_try = first_try.other();

        if self._can_place(word, x, y, intersection_index, &first_try) {
            Some(first_try)
        } else if self._can_place(word, x, y, intersection_index, &second_try) {
            Some(second_try)
        } else {
            None
        }
    }

    fn _can_place(
        &mut self,
        word: &Word,
        x: usize,
        y: usize,
        intersection_index: usize,
        direction: &Direction,
    ) -> bool {
        let last_index = word.text.len() - 1;
        // Is the intersection index out of bounds?
        if intersection_index > last_index {
            return false;
        }

        let origin = if let Direction::Horizontal = direction {
            x
        } else {
            y
        };

        // Would the word go over the top or the left of the puzzle's bounds?
        match intersection_index.cmp(&origin) {
            // Check the space beyond the beginning of the word to make sure it's empty.
            Ordering::Less => {
                let prefix = match direction {
                    Direction::Horizontal => self.puzzle[y].row[origin - intersection_index - 1],
                    Direction::Verticle => self.puzzle[origin - intersection_index - 1].row[x],
                };

                if prefix.is_some() {
                    return false;
                }
            }
            // The word starts at the edge of the board, so no check needed.
            Ordering::Equal => {}
            // The word goes over the edge of the board.
            Ordering::Greater => return false,
        };

        let edge = if let Direction::Horizontal = direction {
            self.width - 1
        } else {
            self.height - 1
        };

        let remainder = last_index - intersection_index;
        // Would the word go over the bottom or the right of the puzzle's bounds?
        match (origin + remainder).cmp(&edge) {
            // Check the space beyond the end of the word to make sure it's empty.
            Ordering::Less => {
                let suffix = match direction {
                    Direction::Horizontal => self.puzzle[y].row[origin + remainder + 1],
                    Direction::Verticle => self.puzzle[origin + remainder + 1].row[x],
                };

                if suffix.is_some() {
                    return false;
                }
            }
            // The word ends at the edge of the board, so no check needed.
            Ordering::Equal => {}
            // The word goes over the edge of the board.
            Ordering::Greater => return false,
        }

        for (letter_count, letter) in word.text.chars().enumerate() {
            let next_index = match letter_count.cmp(&intersection_index) {
                Ordering::Less => origin - (intersection_index - letter_count),
                Ordering::Equal => origin,
                Ordering::Greater => origin + (letter_count - intersection_index),
            };

            let above_or_before = match direction {
                Direction::Horizontal => {
                    if y != 0 {
                        self.puzzle[y - 1].row[next_index]
                    } else {
                        None
                    }
                }
                Direction::Verticle => {
                    if x != 0 {
                        self.puzzle[next_index].row[x - 1]
                    } else {
                        None
                    }
                }
            };

            let below_or_after = match direction {
                Direction::Horizontal => {
                    if y < edge {
                        self.puzzle[y + 1].row[next_index]
                    } else {
                        None
                    }
                }

                Direction::Verticle => {
                    if x < edge {
                        self.puzzle[next_index].row[x + 1]
                    } else {
                        None
                    }
                }
            };

            let space = match direction {
                Direction::Horizontal => self.puzzle[y].row[next_index],
                Direction::Verticle => self.puzzle[next_index].row[x],
            };

            match space {
                Some(c) => {
                    if letter != c {
                        return false;
                    }
                }
                None => {
                    if above_or_before.is_some() {
                        return false;
                    };
                    if below_or_after.is_some() {
                        return false;
                    };
                }
            }
        }

        true
    }
}
