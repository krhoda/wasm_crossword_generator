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

#[cfg(target_arch = "wasm32")]
use tsify::Tsify;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// !!! Building Blocks !!!

// If a word is smaller than three letters, it could potentially break crossword generation.
// Defined as a constant to avoid magic numbers. Callers can provide larger minimums.
const MIN_LETTER_COUNT: usize = 3;

// Word is a record containing a potential portion of the Crossword answer at "text"
// along with an optional "clue" field.
// The "text" field will be split using .chars() with all the implications that brings.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Word {
    pub text: String,
    pub clue: Option<String>,
}

// Direction is used to determine the orientation of the word.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
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

// Placement describes a location on the crossword puzzle. Used to mark word origins.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Placement {
    pub x: usize,
    pub y: usize,
    pub direction: Direction,
}

// PlacedWord represents a word which makes up the crossword puzzle's answers
// The placement field marks the origin and orientation of the word.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PlacedWord {
    pub placement: Placement,
    pub word: Word,
}

// SolutionRow represents of one row of a crossword solution
// Within the interior "row" vector, there is either a None for a blank space
// or a Some(c) where c: char which would be a component of the crossword solution.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct SolutionRow {
    pub row: Vec<Option<char>>,
}

// NOTE: If const generics worked with wasm_bindgen, we could do something way more concise/clear
impl SolutionRow {
    fn new(width: usize) -> SolutionRow {
        let mut row = Vec::<Option<char>>::new();
        for _ in 0..width {
            row.push(None);
        }

        SolutionRow { row }
    }
}

// CrosswordError is what it says on the tin. Often ellided over in favor of retrying, which if
// fails, throws CrosswordError::MaxRetries.
#[derive(Error, Debug)]
pub enum CrosswordError {
    #[error("cannot generate valid puzzle from list of words")]
    BadConfig,
    #[error("word doesn't fit")]
    BadFit,
    #[error("intersection point was empty on non-first word")]
    EmptyIntersection,
    #[error("grid state doesn't match solutions answer array")]
    GridStateError,
    #[error("generated puzzle did not meet requirements")]
    InsufficientPuzzle,
    #[error("player guess word's placement cannot be found")]
    InvalidPlayerGuess,
    #[error("could not generate crossword before hitting max retries")]
    MaxRetries,
    #[error("more answers than words")]
    MoreAnswersThanWords,
    #[error("no valid inital words")]
    NoValidInitialWords,
    #[error("point out of bounds")]
    PointOutOfBounds,
    #[error("word would conflict with other placed word")]
    WordConflict,
    #[error("puzzle doesn't match list of words")]
    WordMismatch,
}

// !!! User Configuration !!!

// CrosswordReqs is a structure holding requirements the final puzzle must meet such as...
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
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
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum CrosswordInitialPlacementStrategy {
    Center(Direction),
    Custom(Placement),
    LowerLeft(Direction),
    LowerRight(Direction),
    UpperLeft(Direction),
    UpperRight(Direction),
}

impl CrosswordInitialPlacementStrategy {
    fn direction(&self) -> Direction {
        match self {
            CrosswordInitialPlacementStrategy::Center(d)
            | CrosswordInitialPlacementStrategy::LowerLeft(d)
            | CrosswordInitialPlacementStrategy::LowerRight(d)
            | CrosswordInitialPlacementStrategy::UpperLeft(d)
            | CrosswordInitialPlacementStrategy::UpperRight(d) => d.clone(),
            CrosswordInitialPlacementStrategy::Custom(p) => p.direction.clone(),
        }
    }
}

impl Distribution<CrosswordInitialPlacementStrategy> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CrosswordInitialPlacementStrategy {
        let direction: Direction = rand::random();
        match rng.gen_range(0..=4) {
            0 => CrosswordInitialPlacementStrategy::Center(direction),
            1 => CrosswordInitialPlacementStrategy::LowerLeft(direction),
            2 => CrosswordInitialPlacementStrategy::LowerRight(direction),
            3 => CrosswordInitialPlacementStrategy::UpperLeft(direction),
            _ => CrosswordInitialPlacementStrategy::UpperRight(direction),
        }
    }
}

impl std::default::Default for CrosswordInitialPlacementStrategy {
    fn default() -> Self {
        rand::random()
    }
}

// CrosswordInitialPlacement allows the caller to specify where and how large the initial word
// placed should be
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct CrosswordInitialPlacement {
    // This differs from CrosswordReq's min_letters_per_word by only applying to the initial word
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

// SolutionConf is the structure used to configure the generation crossword solutions.
#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct SolutionConf {
    // The possible words to use to construct the puzzle
    pub words: Vec<Word>,
    // Maximum words for the puzzle
    pub max_words: usize,
    // How wide the puzzle should be in single letter spaces
    pub width: usize,
    // How tall the puzzle should be in single letter spaces
    pub height: usize,
    // Optional requirements for the puzzle, if a generated puzzle does not meet these
    // then a retry is initiated, the CrosswordReqs structure requires a max_retries field
    // be specified to avoid retrying infinitly
    pub requirements: Option<CrosswordReqs>,
    // Optional requirements for the initial placement, allowing a caller to specify where and
    // how large the inital word placed should be.
    pub initial_placement: Option<CrosswordInitialPlacement>,
}

// !!! Solution Generation and Stateful Puzzles !!!

// Solution represents a complete crossword structure. Does not include stateful
// constructs for user input.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Solution {
    pub grid: Vec<SolutionRow>,
    pub words: Vec<PlacedWord>,
    width: usize,
    height: usize,
}

impl Solution {
    pub fn new(conf: SolutionConf) -> Result<Solution, CrosswordError> {
        if let Ok(crossword) = Solution::generate(conf.clone()) {
            return Ok(crossword);
        } else if let Some(reqs) = &conf.requirements {
            let mut attempt = 0;
            while attempt < reqs.max_retries {
                if let Ok(crossword) = Solution::generate(conf.clone()) {
                    if crossword.is_valid().is_ok() {
                        return Ok(crossword);
                    }
                } else {
                    attempt += 1;
                }
            }
        }

        Err(CrosswordError::MaxRetries)
    }

    // Useful for testing deserialized Solutions from external sources.
    // Also used as a sanity check in the new function
    pub fn is_valid(&self) -> Result<(), CrosswordError> {
        let mut crossword = Solution::new_empty(self.width, self.height);
        for word in self.words.iter() {
            if !crossword.checked_place(
                word.word.clone(),
                word.placement.x,
                word.placement.y,
                0,
                &word.placement.direction,
            ) {
                return Err(CrosswordError::BadConfig);
            };
        }

        if crossword.grid != self.grid {
            return Err(CrosswordError::WordMismatch);
        }

        Ok(())
    }

    fn generate(conf: SolutionConf) -> Result<Solution, CrosswordError> {
        let mut crossword = Solution::new_empty(conf.width, conf.height);

        // Check the config for a min letter count, otherwise set to the constant.
        let min_letter_count = if let Some(reqs) = &conf.requirements {
            match reqs.min_letters_per_word {
                Some(mlc) => {
                    if mlc >= MIN_LETTER_COUNT {
                        mlc
                    } else {
                        MIN_LETTER_COUNT
                    }
                }
                None => MIN_LETTER_COUNT,
            }
        } else {
            MIN_LETTER_COUNT
        };

        let mut words = conf.words;

        // Randomize before the next removal step
        words.shuffle(&mut thread_rng());
        {
            // Open a new scope to drop these vars after using them.
            let mut h = HashSet::new();
            let longest_side = if crossword.width >= crossword.height {
                crossword.width
            } else {
                crossword.height
            };

            // Remove all words under the min letter count
            // Remove all words larger than the largest side.
            // Remove all duplicate words and homographs
            // Because of the above shuffle the homograph retained is random.
            words.retain(|w| {
                w.text.chars().count() >= min_letter_count
                    && w.text.chars().count() <= longest_side
                    && h.insert(w.text.clone())
            });
        }

        // The shadowed version of words will be missing the initially placed word removed
        // from the vector in place.
        let words = crossword.place_initial(&conf.initial_placement, words)?;

        let max_words = conf.max_words;

        for word in words {
            // If max words hit, break.
            if crossword.words.len() >= max_words {
                break;
            }

            // try using each letter in the word to find a valid overlap on the puzzle.
            for (intersection_index, letter) in word.text.chars().enumerate() {
                // Was the word placed at the using current intersection index?
                if Solution::place_word(&mut crossword, intersection_index, &word, &letter)
                    .is_some()
                {
                    // If so, try to place next word.
                    break;
                }
                // If not, try the next letter in this word
            }
        }

        // Once the words vector is exhausted, check the resulting puzzle against the (optional)
        // requirements porition of the config
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
                } else {
                    Err(CrosswordError::InsufficientPuzzle)
                }
            }
            None => Ok(crossword),
        }
    }

    fn place_initial(
        &mut self,
        initial_placement_conf: &Option<CrosswordInitialPlacement>,
        words: Vec<Word>,
    ) -> Result<Vec<Word>, CrosswordError> {
        // Extract the initial placement instructions or generate them from defaults
        // if not provided
        let (min_letter_count, strategy) = match initial_placement_conf {
            None => {
                let def = CrosswordInitialPlacement::default();
                // Unwraps are safe since it's been constructed from default.
                (def.min_letter_count.unwrap(), def.strategy.unwrap())
            }
            Some(ip) => {
                let initial_mlc = if let Some(mlc) = ip.min_letter_count {
                    mlc
                } else {
                    MIN_LETTER_COUNT
                };

                let initial_strat = if let Some(strat) = ip.strategy.clone() {
                    strat
                } else {
                    CrosswordInitialPlacementStrategy::default()
                };
                (initial_mlc, initial_strat)
            }
        };

        let mut words = words;
        // Create a place to hold words whose minimum letter count is greater than the puzzle's
        // minimum but less than the initial word's minimum. These will be appended back onto the
        // "words"
        let mut skipped_words: Vec<Word> = Vec::new();
        match words.pop() {
            None => Err(CrosswordError::NoValidInitialWords),
            Some(word) => {
                let mut word = word;
                loop {
                    match self.try_initial_placement(min_letter_count, &strategy, word) {
                        Ok(_) => {
                            break;
                        }

                        Err(w) => {
                            skipped_words.push(w);

                            match words.pop() {
                                None => {
                                    return Err(CrosswordError::NoValidInitialWords);
                                }
                                Some(w) => {
                                    word = w;
                                }
                            }
                        }
                    };
                }

                skipped_words.reverse();
                words.append(&mut skipped_words);
                Ok(words)
            }
        }
    }

    // returns true if the word was placed, false if it fails to meet some requirement
    fn try_initial_placement(
        &mut self,
        min_letter_count: usize,
        strategy: &CrosswordInitialPlacementStrategy,
        word: Word,
    ) -> Result<(), Word> {
        let max_letter_count = match strategy.direction() {
            Direction::Horizontal => self.width,
            Direction::Verticle => self.height,
        };

        let count = word.text.chars().count();
        if count < min_letter_count && count > max_letter_count {
            return Err(word);
        };

        // To avoid magic numbers:
        let max_x = self.width - 1;
        let max_y = self.height - 1;
        let last_index = count - 1;

        match strategy {
            CrosswordInitialPlacementStrategy::Center(direction) => {
                // This gets a little tricky, but basically if the height / width / count
                // are even, then a choice between mid points occurs. Because of zero indexing,
                // this is expressed as a 50% chance of a -1 to the midpoint. The underlying
                // word uses the offset matching it's orientaion. Because of the earlier math
                // around max letter count, we know this won't go out of bounds.

                let rem_y = self.height % 2;
                let y_offset = if rem_y == 0 {
                    thread_rng().gen_range(0..=1)
                } else {
                    0
                };

                let mid_y = if rem_y == 0 {
                    (self.height / 2) - y_offset
                } else {
                    self.height / 2
                };

                let rem_x = self.width % 2;
                let x_offset = if rem_x == 0 {
                    thread_rng().gen_range(0..=1)
                } else {
                    0
                };

                let mid_x = if rem_y == 0 {
                    (self.height / 2) - x_offset
                } else {
                    self.height / 2
                };

                let w_rem = count % 2;

                let w_mid = match direction {
                    Direction::Horizontal => {
                        if w_rem == 0 {
                            (count / 2) - x_offset
                        } else {
                            count / 2
                        }
                    }
                    Direction::Verticle => {
                        if w_rem == 0 {
                            (count / 2) - y_offset
                        } else {
                            count / 2
                        }
                    }
                };
                self._unchecked_place(word, mid_x, mid_y, w_mid, direction);
                Ok(())
            }
            CrosswordInitialPlacementStrategy::Custom(placement) => {
                if placement.x >= self.width {
                    return Err(word);
                }

                if placement.y >= self.height {
                    return Err(word);
                }

                match placement.direction {
                    Direction::Horizontal => {
                        if placement.x + count > self.width {
                            return Err(word);
                        }
                    }
                    Direction::Verticle => {
                        if placement.y + count > self.height {
                            return Err(word);
                        }
                    }
                }

                self._unchecked_place(word, placement.x, placement.y, 0, &placement.direction);

                Ok(())
            }
            CrosswordInitialPlacementStrategy::LowerLeft(direction) => {
                match direction {
                    Direction::Horizontal => {
                        self._unchecked_place(word, 0, max_y, 0, direction);
                    }
                    Direction::Verticle => {
                        self._unchecked_place(word, 0, max_y, last_index, direction);
                    }
                }

                Ok(())
            }
            CrosswordInitialPlacementStrategy::LowerRight(direction) => {
                self._unchecked_place(word, max_x, max_y, last_index, direction);

                Ok(())
            }
            CrosswordInitialPlacementStrategy::UpperLeft(direction) => {
                self._unchecked_place(word, 0, 0, 0, direction);
                Ok(())
            }
            CrosswordInitialPlacementStrategy::UpperRight(direction) => {
                match direction {
                    Direction::Horizontal => {
                        self._unchecked_place(word, max_x, 0, last_index, direction);
                    }
                    Direction::Verticle => {
                        self._unchecked_place(word, max_x, 0, 0, direction);
                    }
                }
                Ok(())
            }
        }
    }

    // This will return Some(()) if the letter was placed.
    fn place_word(
        crossword: &mut Solution,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        let temp = crossword.clone();
        for (row_count, _) in temp.grid.iter().enumerate() {
            if Solution::place_on_row(crossword, row_count, intersection_index, word, letter)
                .is_some()
            {
                return Some(());
            }
        }

        None
    }

    // This will return Some(()) if the letter was placed.
    fn place_on_row(
        crossword: &mut Solution,
        row_count: usize,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        let temp = crossword.clone();
        for (col_count, _) in temp.grid[row_count].row.iter().enumerate() {
            if Solution::place_on_column(
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
        crossword: &mut Solution,
        col_count: usize,
        row_count: usize,
        intersection_index: usize,
        word: &Word,
        letter: &char,
    ) -> Option<()> {
        if let Some(c) = crossword.grid[row_count].row[col_count] {
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
    fn new_empty(width: usize, height: usize) -> Solution {
        let mut grid = Vec::<SolutionRow>::new();

        for _ in 0..height {
            grid.push(SolutionRow::new(width))
        }

        Solution {
            grid,
            words: Vec::<PlacedWord>::new(),
            width,
            height,
        }
    }

    // returns a count of completely empty columns
    fn empty_columns(&self) -> usize {
        let mut acc = 0;
        for i in 0..self.width {
            let mut is_empty = true;
            for row in self.grid.clone() {
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
        for row in self.grid.clone() {
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
    fn place(
        &mut self,
        word: Word,
        x: usize,
        y: usize,
        intersection_index: usize,
    ) -> Result<(), CrosswordError> {
        if x >= self.width || y >= self.height {
            return Err(CrosswordError::PointOutOfBounds);
        }

        // Can the word be placed at these coordinates? If so, which direction?
        // In the case that either direction is valid, Cthulu has risen, but also,
        // it would tolerare this non-Eucludian condition and pick randomly.
        let placement_direction = self.can_place(&word, x, y, intersection_index);

        if let Some(direction) = placement_direction {
            self._unchecked_place(word, x, y, intersection_index, &direction);
            Ok(())
        } else {
            Err(CrosswordError::BadFit)
        }
    }

    // _unchecked_place bypasses any checks on validity and is only useful for the
    // initial placement
    fn _unchecked_place(
        &mut self,
        word: Word,
        x: usize,
        y: usize,
        intersection_index: usize,
        direction: &Direction,
    ) {
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
                    self.grid[y].row[next_index] = Some(letter);
                }
                Direction::Verticle => {
                    self.grid[next_index].row[x] = Some(letter);
                }
            };
        }

        let placement_x = match direction {
            Direction::Horizontal => x - intersection_index,
            Direction::Verticle => x,
        };

        let placement_y = match direction {
            Direction::Horizontal => y,
            Direction::Verticle => y - intersection_index,
        };

        let placement = Placement {
            direction: direction.clone(),
            x: placement_x,
            y: placement_y,
        };

        self.words.push(PlacedWord { word, placement });
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

        // Would the word go over the top or the left of the grid's bounds?
        match intersection_index.cmp(&origin) {
            // Check the space beyond the beginning of the word to make sure it's empty.
            Ordering::Less => {
                let prefix = match direction {
                    Direction::Horizontal => self.grid[y].row[origin - intersection_index - 1],
                    Direction::Verticle => self.grid[origin - intersection_index - 1].row[x],
                };

                if prefix.is_some() {
                    return false;
                }
            }
            // The word starts at the edge of the board, so no check needed.
            Ordering::Equal => {}
            // The word goes over the edge of the board.
            Ordering::Greater => return false,
        }

        let edge = if let Direction::Horizontal = direction {
            self.width - 1
        } else {
            self.height - 1
        };

        let remainder = last_index - intersection_index;
        // Would the word go over the bottom or the right of the grid's bounds?
        match (origin + remainder).cmp(&edge) {
            // Check the space beyond the end of the word to make sure it's empty.
            Ordering::Less => {
                let suffix = match direction {
                    Direction::Horizontal => self.grid[y].row[origin + remainder + 1],
                    Direction::Verticle => self.grid[origin + remainder + 1].row[x],
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

        // Is there correct spacing around the word?
        for (letter_count, letter) in word.text.chars().enumerate() {
            let next_index = match letter_count.cmp(&intersection_index) {
                Ordering::Less => origin - (intersection_index - letter_count),
                Ordering::Equal => origin,
                Ordering::Greater => origin + (letter_count - intersection_index),
            };

            let above_or_before = match direction {
                Direction::Horizontal => {
                    if y != 0 {
                        self.grid[y - 1].row[next_index]
                    } else {
                        None
                    }
                }
                Direction::Verticle => {
                    if x != 0 {
                        self.grid[next_index].row[x - 1]
                    } else {
                        None
                    }
                }
            };

            let below_or_after = match direction {
                Direction::Horizontal => {
                    if y < edge {
                        self.grid[y + 1].row[next_index]
                    } else {
                        None
                    }
                }

                Direction::Verticle => {
                    if x < edge {
                        self.grid[next_index].row[x + 1]
                    } else {
                        None
                    }
                }
            };

            let space = match direction {
                Direction::Horizontal => self.grid[y].row[next_index],
                Direction::Verticle => self.grid[next_index].row[x],
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

        // Is there an already placed word this would over-write?
        for placed_word in self.words.iter() {
            if &placed_word.placement.direction == direction {
                match direction {
                    Direction::Horizontal => {
                        if placed_word.placement.y == y {
                            let fst_index = x - intersection_index;
                            let lst_index = fst_index + (word.text.chars().count() - 1);

                            let current_fst = placed_word.placement.x;
                            let current_lst =
                                current_fst + (placed_word.word.text.chars().count() - 1);

                            if fst_index <= current_lst && lst_index >= current_fst {
                                return false;
                            }
                        }
                    }
                    Direction::Verticle => {
                        if placed_word.placement.x == x {
                            let fst_index = y - intersection_index;
                            let lst_index = fst_index + (word.text.chars().count() - 1);

                            let current_fst = placed_word.placement.y;
                            let current_lst =
                                current_fst + (placed_word.word.text.chars().count() - 1);

                            if fst_index <= current_lst && lst_index >= current_fst {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    // Used for testing validity for structs that are supplied externally.
    fn checked_place(
        &mut self,
        word: Word,
        x: usize,
        y: usize,
        intersection_index: usize,
        direction: &Direction,
    ) -> bool {
        if self._can_place(&word, x, y, intersection_index, direction) {
            self._unchecked_place(word, x, y, intersection_index, direction);
            true
        } else {
            false
        }
    }
}

// NOTE: This could be represented as just a Option<Option<char>> but that will get ambiguous
// serializing / deserailizing to JS.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PuzzleSpace {
    char_slot: Option<char>,
    has_char_slot: bool,
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PuzzleRow {
    pub row: Vec<PuzzleSpace>,
}

// Puzzle is a stateful game composed of a solution and a list of answers.
#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Puzzle {
    pub solution: Solution,
    pub player_answers: Vec<PlacedWord>,
    pub grid: Vec<PuzzleRow>,
}

impl Puzzle {
    fn new_grid(grid: &[SolutionRow]) -> Vec<PuzzleRow> {
        let mut result: Vec<PuzzleRow> = Vec::new();

        for _ in grid.iter() {
            result.push(PuzzleRow { row: Vec::new() });
        }

        for (y, row) in grid.iter().enumerate() {
            for space in row.row.iter() {
                let next = PuzzleSpace {
                    has_char_slot: space.is_some(),
                    char_slot: None,
                };

                result[y].row.push(next);
            }
        }

        result
    }

    pub fn grid_from_answers(&mut self) -> Result<(), CrosswordError> {
        self.grid = Puzzle::new_grid(&self.solution.grid);
        let answers = self.player_answers.clone();
        for word in answers.iter() {
            match self.check_place_answer(word) {
                Ok(Some(_)) => {
                    self.place_answer_on_grid(word);
                }
                _ => {
                    self.grid = Puzzle::new_grid(&self.solution.grid);
                    return Err(CrosswordError::GridStateError);
                }
            }
        }
        Ok(())
    }

    pub fn new(conf: SolutionConf) -> Result<Puzzle, CrosswordError> {
        let solution = Solution::new(conf)?;
        let grid: Vec<PuzzleRow> = Puzzle::new_grid(&solution.grid);

        Ok(Puzzle {
            solution,
            player_answers: Vec::new(),
            grid,
        })
    }

    fn check_place_answer(&self, placed_word: &PlacedWord) -> Result<Option<()>, CrosswordError> {
        match self
            .solution
            .words
            .iter()
            .find(|w| w.placement == placed_word.placement)
        {
            Some(w) => {
                if w.word.text.len() != placed_word.word.text.len() {
                    return Ok(None);
                }

                match placed_word.placement.direction {
                    Direction::Horizontal => {
                        let y = placed_word.placement.y;
                        for (word_index, next_char) in placed_word.word.text.chars().enumerate() {
                            let x = word_index + placed_word.placement.x;
                            if !self.grid[y].row[x].has_char_slot {
                                return Err(CrosswordError::GridStateError);
                            }

                            if let Some(current_char) = self.grid[y].row[x].char_slot {
                                if current_char != next_char {
                                    return Ok(None);
                                }
                            };
                        }
                    }
                    Direction::Verticle => {
                        let x = placed_word.placement.x;
                        for (word_index, next_char) in placed_word.word.text.chars().enumerate() {
                            let y = word_index + placed_word.placement.y;
                            if !self.grid[y].row[x].has_char_slot {
                                return Err(CrosswordError::GridStateError);
                            }

                            if let Some(current_char) = self.grid[y].row[x].char_slot {
                                if current_char != next_char {
                                    return Ok(None);
                                }
                            };
                        }
                    }
                }
            }
            None => return Err(CrosswordError::InvalidPlayerGuess),
        };

        Ok(Some(()))
    }

    // If the word fits, adds the word to the player_answers and grid fields.
    // If there is a conflict or the word is the wrong length returns an Ok(None).
    // Returns error if guess is invalid, meaning the placement isn't in the solutions vec
    pub fn place_answer(&mut self, placed_word: PlacedWord) -> Result<Option<()>, CrosswordError> {
        match self.check_place_answer(&placed_word)? {
            Some(_) => {
                self.place_answer_on_grid(&placed_word);
                self.player_answers.push(placed_word);
                Ok(Some(()))
            }
            None => Ok(None),
        }
    }

    fn place_answer_on_grid(&mut self, placed_word: &PlacedWord) {
        match placed_word.placement.direction {
            Direction::Horizontal => {
                let y = placed_word.placement.y;
                let x_base = placed_word.placement.x;
                for (word_index, next_char) in placed_word.word.text.chars().enumerate() {
                    let x = x_base + word_index;
                    self.grid[y].row[x].char_slot = Some(next_char);
                }
            }

            Direction::Verticle => {
                let x = placed_word.placement.x;
                let y_base = placed_word.placement.y;
                for (word_index, next_char) in placed_word.word.text.chars().enumerate() {
                    let y = y_base + word_index;
                    self.grid[y].row[x].char_slot = Some(next_char);
                }
            }
        }
    }

    pub fn remove_answer(&mut self, placement: &Placement) {
        self.player_answers.retain(|w| &w.placement != placement)
    }

    pub fn is_complete(&self) -> Result<bool, CrosswordError> {
        match self.player_answers.len().cmp(&self.solution.words.len()) {
            // The player must supply more answers
            Ordering::Less => Ok(false),
            // The puzzle is complete
            Ordering::Equal => Ok(self
                .player_answers
                .iter()
                .all(|word| self.solution.words.contains(word))),
            // The number of submitted answers is greater than the total answers
            Ordering::Greater => Err(CrosswordError::MoreAnswersThanWords),
        }
    }

    // Returns all user answers that are incorrect, along with the coresponding correction
    pub fn wrong_answers_and_solutions(
        &self,
    ) -> Result<Vec<(PlacedWord, PlacedWord)>, CrosswordError> {
        let mut result: Vec<(PlacedWord, PlacedWord)> = Vec::new();
        for word in self.player_answers.iter() {
            if !self.solution.words.contains(word) {
                let correction = if let Some(c) = self
                    .solution
                    .words
                    .iter()
                    .find(|w| w.placement == word.placement)
                {
                    c.clone()
                } else {
                    return Err(CrosswordError::InvalidPlayerGuess);
                };
                result.push((word.clone(), correction));
            }
        }

        Ok(result)
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum GuessResult {
    Conflict,
    Complete,
    Correct,
    InvalidPlacement,
    InvalidTooManyAnswers,
    Repeat,
    StateError,
    Unchecked,
    Wrong,
}

pub trait Playmode
where
    Self: Sized,
{
    fn guess_word(&mut self, word: PlacedWord) -> GuessResult;
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct ClassicPuzzle {
    pub puzzle: Puzzle,
}

impl ClassicPuzzle {
    pub fn new(conf: SolutionConf) -> Result<ClassicPuzzle, CrosswordError> {
        Ok(ClassicPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }

    pub fn remove_answer(&mut self, placement: &Placement) {
        self.puzzle.remove_answer(placement)
    }
}

impl Playmode for ClassicPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> GuessResult {
        // Is this a valid guess?
        if !self
            .puzzle
            .solution
            .words
            .iter()
            .any(|w| w.placement == word.placement)
        {
            return GuessResult::InvalidPlacement;
        }

        // Remove previous guess at placement if exists
        self.puzzle
            .player_answers
            .retain(|w| w.placement != word.placement);

        // Remove the previous guess from the grid
        if self.puzzle.grid_from_answers().is_err() {
            return GuessResult::StateError;
        }

        match self.puzzle.place_answer(word) {
            // This branch executes if the puzzle state is bad or the placement of the guess is unexpected
            Err(e) => match e {
                CrosswordError::InvalidPlayerGuess => GuessResult::InvalidPlacement,
                _ => GuessResult::StateError,
            },
            // This branch executes if the placement is valid, even if the word still conflicts
            Ok(option) => match option {
                Some(_) => GuessResult::Unchecked,
                None => GuessResult::Conflict,
            },
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PlacedWordPuzzle {
    pub puzzle: Puzzle,
}

impl PlacedWordPuzzle {
    pub fn new(conf: SolutionConf) -> Result<PlacedWordPuzzle, CrosswordError> {
        Ok(PlacedWordPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }
}

impl Playmode for PlacedWordPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> GuessResult {
        match self.puzzle.is_complete() {
            Ok(b) => {
                if b {
                    return GuessResult::Complete;
                }
            }
            Err(_) => return GuessResult::InvalidTooManyAnswers,
        }

        let correct = if let Some(c) = self
            .puzzle
            .solution
            .words
            .iter()
            .find(|w| w.placement == word.placement)
        {
            c.clone()
        } else {
            return GuessResult::InvalidPlacement;
        };

        if correct == word {
            match self.puzzle.place_answer(word) {
                // This branch executes if the puzzle state is bad or the placement of the guess is unexpected
                Err(e) => match e {
                    CrosswordError::InvalidPlayerGuess => GuessResult::InvalidPlacement,
                    _ => GuessResult::StateError,
                },
                // This branch executes if the placement is valid, even if the word still conflicts
                Ok(option) => match option {
                    Some(_) => match self.puzzle.is_complete() {
                        Ok(b) => {
                            if b {
                                GuessResult::Complete
                            } else {
                                GuessResult::Correct
                            }
                        }
                        Err(_) => GuessResult::InvalidTooManyAnswers,
                    },
                    None => GuessResult::Conflict,
                },
            }
        } else {
            GuessResult::Wrong
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    target_arch = "wasm32",
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PerWordPuzzle {
    pub puzzle: Puzzle,
}

impl PerWordPuzzle {
    pub fn new(conf: SolutionConf) -> Result<PerWordPuzzle, CrosswordError> {
        Ok(PerWordPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }
}

impl Playmode for PerWordPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> GuessResult {
        match self.puzzle.is_complete() {
            Ok(b) => {
                if b {
                    return GuessResult::Complete;
                };
            }
            Err(_) => return GuessResult::InvalidTooManyAnswers,
        }

        if self
            .puzzle
            .player_answers
            .iter()
            .any(|w| w.word.text == word.word.text)
        {
            GuessResult::Repeat
        } else if let Some(c) = self
            .puzzle
            .solution
            .words
            .iter()
            .find(|w| w.word.text == word.word.text)
        {
            match self.puzzle.place_answer(c.clone()) {
                // This branch executes if the puzzle state is bad or the placement of the guess is unexpected
                Err(e) => match e {
                    CrosswordError::InvalidPlayerGuess => GuessResult::InvalidPlacement,
                    _ => GuessResult::StateError,
                },
                // This branch executes if the placement is valid, even if the word still conflicts
                Ok(option) => match option {
                    Some(_) => match self.puzzle.is_complete() {
                        Ok(b) => {
                            if b {
                                GuessResult::Complete
                            } else {
                                GuessResult::Correct
                            }
                        }
                        Err(_) => GuessResult::InvalidTooManyAnswers,
                    },
                    None => GuessResult::Conflict,
                },
            }
        } else {
            GuessResult::Wrong
        }
    }
}

// !!! WASM Specific Stuff !!!

/* Uncomment for WASM in-browser debug
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
*/

#[cfg(target_arch = "wasm32")]
#[allow(clippy::from_over_into)]
impl std::convert::Into<JsValue> for CrosswordError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

// This is the only way JS/WASM applications can construct Solution structs
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn new_solution(conf: SolutionConf) -> Result<Solution, CrosswordError> {
    Solution::new(conf)
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PuzzleType {
    Classic,
    PlacedWord,
    PerWord,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleContainer {
    puzzle_type: PuzzleType,
    puzzle: Puzzle,
}

// This is the only way JS/WASM applications can construct Puzzle structs
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn new_puzzle(
    conf: SolutionConf,
    puzzle_type: PuzzleType,
) -> Result<PuzzleContainer, CrosswordError> {
    Ok(PuzzleContainer {
        puzzle_type,
        puzzle: Puzzle::new(conf)?,
    })
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleCompleteContainer {
    pub puzzle_container: PuzzleContainer,
    pub is_complete: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn is_puzzle_complete(
    puzzle_container: PuzzleContainer,
) -> Result<PuzzleCompleteContainer, CrosswordError> {
    let is_complete = puzzle_container.puzzle.is_complete()?;
    Ok(PuzzleCompleteContainer {
        puzzle_container,
        is_complete,
    })
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WrongAnswerPair {
    pub got: PlacedWord,
    pub wanted: PlacedWord,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WrongAnswersContainer {
    pub puzzle_container: PuzzleContainer,
    pub wrong_answer_pairs: Vec<WrongAnswerPair>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wrong_answers_and_solutions(
    puzzle_container: PuzzleContainer,
) -> Result<WrongAnswersContainer, CrosswordError> {
    let wrong_answer_pairs = puzzle_container
        .puzzle
        .wrong_answers_and_solutions()?
        .into_iter()
        .map(|(got, wanted)| WrongAnswerPair { got, wanted })
        .collect();

    Ok(WrongAnswersContainer {
        puzzle_container,
        wrong_answer_pairs,
    })
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleAndResult {
    puzzle_container: PuzzleContainer,
    guess_result: GuessResult,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn guess_word(puzzle_container: PuzzleContainer, guess: PlacedWord) -> PuzzleAndResult {
    let puzzle_type = puzzle_container.puzzle_type;
    let (puzzle, guess_result) = match puzzle_type {
        PuzzleType::Classic => {
            let mut classic = ClassicPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = classic.guess_word(guess);
            (classic.puzzle, res)
        }
        PuzzleType::PlacedWord => {
            let mut placed_word = PlacedWordPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = placed_word.guess_word(guess);
            (placed_word.puzzle, res)
        }
        PuzzleType::PerWord => {
            let mut per_word = PerWordPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = per_word.guess_word(guess);
            (per_word.puzzle, res)
        }
    };

    PuzzleAndResult {
        puzzle_container: PuzzleContainer {
            puzzle_type,
            puzzle,
        },
        guess_result,
    }
}

// This is a debug feature that is called from <repo>/src/crossword_gen_wrapper.ts
// It improves the quality of error messages that are printed to the dev console
// For more details see
// https://github.com/rustwasm/console_error_panic_hook#readme
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
