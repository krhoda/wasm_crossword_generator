mod utils;

use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    thread_rng, Rng,
};

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::cmp::{Ord, Ordering};
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

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Word {
    pub text: String,
    pub clue: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Direction {
    Horizontal,
    Verticle,
}

impl Direction {
    pub fn other(&self) -> Direction {
        match self {
            Direction::Horizontal => Direction::Verticle,
            Direction::Verticle => Direction::Horizontal,
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=1) {
            0 => Direction::Horizontal,
            _ => Direction::Verticle,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PlacedWord {
    // TODO: Add x / y coords
    pub direction: Direction,
    pub word: Word,
}

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

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Placement {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Crossword {
    pub puzzle: Vec<CrosswordRow>,
    pub words: Vec<PlacedWord>,
    width: usize,
    height: usize,
}

#[derive(Error, Debug)]
pub enum CrosswordError {
    #[error("word doesn't fit")]
    BadFit,
    #[error("intersection point can only be empty for placement of first word")]
    EmptyIntersection,
    #[error("could not generate crossword before hitting max retries")]
    MaxRetries,
    #[error("point out of bounds")]
    PointOutOfBounds,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordConf {
    pub words: Vec<Word>,
    pub max_words: usize,
    pub width: usize,
    pub height: usize,
    pub requirements: Option<CrosswordReqs>,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordReqs {
    pub max_retries: usize,
    pub min_words: Option<usize>,
    pub max_empty_columns: Option<usize>,
    pub max_empty_rows: Option<usize>,
}

#[wasm_bindgen]
pub fn new_crossword(conf: &str) -> Result<Crossword, JsError> {
    set_panic_hook();
    let conf = from_str::<CrosswordConf>(conf).map_err(|e| JsError::new(&e.to_string()))?;
    Crossword::new(conf).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
impl Crossword {
    fn new(conf: CrosswordConf) -> Result<Crossword, CrosswordError> {
        Crossword::new_recursive(conf, 0)
    }

    fn new_recursive(conf: CrosswordConf, iteration: usize) -> Result<Crossword, CrosswordError> {
        // To avoid issues with borrowing later.
        let c = conf.clone();

        let mut crossword = Crossword::new_empty(conf.width, conf.height);

        let mut words = conf.words;
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
                // Work from here
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

    fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

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
    // The first word is special cased.
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
