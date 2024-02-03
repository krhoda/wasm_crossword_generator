//! #  WASM Crossword Generator
//!
//! `wasm_crossword_generator` is a library for generating and operating crossword puzzles with
//! first-class WebAssembly (WASM) support targeting the browser and other WASM environments. While
//! fully functional and ergonomic as a Rust library, the design is WASM-first so some trade-offs are
//! made such as not using const generics, avoiding `Option<Option<T>>` because of ambiguity during
//! JSON de/serialization, and using structs over tuples.
//!
//! This library exposes configuration options for specifying the size and density of the crossword.
//! Three stateful implementations of [Playmodes](Playmode) are also supported to facilitate different styles of puzzle.
//!
//! The most basic example of usage in pure Rust looks like:
//! ```rust
//! # fn main() -> Result<(), wasm_crossword_generator::CrosswordError> {
//! use wasm_crossword_generator::*;
//!
//! let words: Vec<Word> = vec![
//!   Word { text: "library".to_string(), clue: Some("Not a framework.".to_string()) },
//!   // In a real usage, there would be more entries.
//! ];
//!
//! // A real SolutionConf would probably want "requirements" to allow for retrying crossword
//! // generation. Because there is only one word, we know we'll get the world's simplest puzzle in
//! // one try.
//! let solution_conf = SolutionConf {
//!     words: words,
//!     max_words: 20,
//!     width: 10,
//!     height: 10,
//!     requirements: None,
//!     initial_placement: None,
//! };
//!
//! // A PerWordPuzzle is one where the player only guesses words, not locations. The player is
//! // immediately informed if the guess is correct or not.
//! let mut puzzle = PerWordPuzzle::new(solution_conf)?;
//!
//! let guess = PlacedWord {
//!   // Because it is a PerWordPuzzle, the placement is ignored, unlike other Playmodes.
//!   placement: Placement { x: 0, y: 0, direction: rand::random() },
//!
//!   // NOTE: you don't need to match the "clue" field, it is ignored for purposes of PartialEq
//!   word: Word { text: "library".to_string(), clue: None }
//! };
//!
//! let guess_result = puzzle.guess_word(guess)?;
//!
//! // Because there is only one word, the guess will result in "Complete" instead of "Correct"
//! assert_eq!(guess_result, GuessResult::Complete);
//! # Ok(())
//! # }
//!
//! ```
//! A quick tour of the pure Rust structures looks like:
//!
//! [Word] a structure containing the field `text` (the string whose characters make up the answer)
//! and an optional, self-descriptive `clue` field.
//!
//! [Direction] is an enum of either `Verticle` or `Horizontal`, used to describe a `Word`'s
//! orientation.
//!
//! [Placement] is a struct containing a pair of coordinates (the fields `x` and `y`) and a
//! `direction` field of type `Direction`.
//!
//! [PlacedWord] is a combination of a `Word` (field: `word`) and `Placement` (field: `placement`).
//! This is used for the internal representation of the `Solution` in it's list of answers.
//!
//! [SolutionRow] is a struct with a `row` field of type `Vec<Option<char>>`. This is used to
//! to represent a row of the crossword, with each item being of type `Some(c: char)` in the case
//! of the square being part of a solution or `None` in the case that the space is blank.
//!
//! [Solution] is a struct with a `grid` field that is a `Vec<SolutionRow>` and a `words` field
//! that is a `Vec<PlacedWord>`, along with a `width` and `height`. This is used by the puzzle
//! structs to check player answers and build the stateful game.
//!
//! [SolutionConf] is a struct containing a set of options for generating [Solutions](Solution).
//! It includes optional sub-structs [CrosswordReqs] and [CrosswordInitialPlacement] among several
//! other fields.
//!
//! [PuzzleSpace] is a struct that represents a stateful space in a game. Contains a bool indicating
//! whether there would be a letter if the puzzle were solved, and a `Option<char>` representing
//! whether a guess has been made or not.
//!
//! [PuzzleRow] is similiar to [SolutionRow], but the `row` field is a `Vec<PuzzleSpace>`.
//!
//! [Puzzle] is a struct representing a stateful crossword puzzle game that responds to user input.
//! It containes a `solution` field with it's corresponding [Solution], a `player_answers` field that
//! is a `Vec<PlacedWord>` representing the player's (not neccessarily correct) answers, and `grid`
//! field similar to the one found in [Solution], but this time of type `Vec<PuzzleRow>`.
//!
//! Three [Playmodes](Playmode) exist which make use of the [Puzzle] struct as their internal state:
//! [ClassicPuzzle] -- a clue-based crossword that doesn't tell the user if their guess is correct,
//! [PlacedWordPuzzle] -- a puzzle where the player specifies a PlacedWord guess and is told if the
//! guess was correct, and [PerWordPuzzle] where the player simply guesses a word and if it is present
//! in the puzzle, the player is told that the guess is correct.
//!
//! Finally, the [GuessResult] enum encapsulates the possible results from all [Playmodes](Playmode).
//!
//! Additionally, there are some types used specifically for WASM-based scenerios, mostly wrapper
//! types, specifiers, and types used to return data passed in from the caller back to the caller.
//! These include: [PuzzleType], [PuzzleContainer], [PuzzleCompleteContainer], [WrongAnswerPair],
//! [WrongAnswersContainer], and [PuzzleAndResult].
//!
//! See [the project repo](https://github.com/krhoda/wasm_crossword_generator) for more details
//! and instructions on how to run examples. A specifically WASM-based example is found [here](https://github.com/krhoda/wasm_crossword_generator/tree/main/example/react_web_example).

#![warn(missing_docs)]

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

#[cfg(any(doc, target_arch = "wasm32"))]
use tsify::Tsify;
#[cfg(any(doc, target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

// !!! Building Blocks !!!

/// Used to set the minimum size in chars of words that act as solution answers.
/// If a word is smaller than three letters, it could potentially break crossword generation.
/// Defined as a constant to avoid magic numbers. Callers can provide larger minimums.
const MIN_LETTER_COUNT: usize = 3;

/// Word is a record containing a potential portion of the Crossword answer at "text"
/// along with an optional "clue" field.
/// The "text" field will be split using .chars() with all the implications that brings.
#[derive(Clone, Debug, Deserialize, Eq, Serialize)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Word {
    /// The literal chars that make up the answer word in string format.
    pub text: String,
    /// The optional clue to be displayed in some crossword game formats.
    pub clue: Option<String>,
}

/// This implementation is done so that guesses do not need to have the same clue entry as the
/// solution.
impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

/// Direction is used to determine the orientation of the word. Includes an "other" function to get
/// a given direction's inverse and impls Distribution (50/50) so that it can be generated using
/// the local RNG.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum Direction {
    /// Horizontal indicates the left-to-right orientation
    Horizontal,
    /// Verticle indicates the up-to-down orientation
    Verticle,
}

impl Direction {
    /// other is logical "not" for Direction
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

/// Placement describes a location on the crossword puzzle. Used to mark word origins.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Placement {
    /// x is one of the Placement's coordinates
    pub x: usize,
    /// y is one of the Placement's coordinates
    pub y: usize,
    /// direction describes the orientation of the Placement
    pub direction: Direction,
}

/// PlacedWord represents a word which makes up the crossword puzzle's answers
/// The placement field marks the origin and orientation of the word.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PlacedWord {
    /// placement contains the coordinates of origin and orientation of the PlacedWord
    pub placement: Placement,
    /// word contains the text of the solution and possibly the clue
    pub word: Word,
}

/// SolutionRow represents of one row of a crossword solution.
/// Within the interior "row" vector, there is either a None for a blank space
/// or a Some(c) where c: char which would be a component of the crossword solution.
/// Constructed by passing in a "width" param.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct SolutionRow {
    /// row is a vector of `Option<char>` with None representing an empty space and Some(char)
    /// representing part of the solution's words.
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

/// CrosswordError is what it says on the tin. In generation, these are ellided over in favor
/// of retrying, which if fails, bubbles the CrosswordError::MaxRetries error.
#[derive(Error, Debug)]
pub enum CrosswordError {
    /// BadConfig is when a valid puzzle cannot be generated from the configuration.
    #[error("cannot generate valid puzzle from list of words")]
    BadConfig,
    /// BadFit describes a word that doesn't fit at a given Placement.
    #[error("word doesn't fit")]
    BadFit,
    /// BadPuzzleType describes when a bad command is issued by an external client.
    #[error("invalid operation for given PuzzleType")]
    BadPuzzleType,
    /// EmptyIntersection describes when a word is placed incorrectly during generation.
    #[error("intersection point was empty on non-first word")]
    EmptyIntersection,
    /// GridStateError describes when the puzzles internal state is incoherent at runtime.
    #[error("grid state doesn't match solutions answer array")]
    GridStateError,
    /// InsufficientPuzzle describes when the generated puzzle fails to meet the given requirements.
    #[error("generated puzzle did not meet requirements")]
    InsufficientPuzzle,
    /// InvalidPlayerGuess describes when a bad guess is given.
    #[error("player guess word's placement cannot be found")]
    InvalidPlayerGuess,
    /// MaxRetries occurs when retrying crossword generation fails equal to the max_retries option.
    #[error("could not generate crossword before hitting max retries")]
    MaxRetries,
    /// MoreAnswersThanWords occurs when puzzle state is incoherent at runtime.
    #[error("more answers than words")]
    MoreAnswersThanWords,
    /// NoValidInitialWords occurs when inital word requirements have no qualifying word in the words
    /// vector.
    #[error("no valid inital words")]
    NoValidInitialWords,
    /// PointOutOfBounds occurs when a given Placement's coordinates are out of bounds from the
    /// Solution's grid
    #[error("point out of bounds")]
    PointOutOfBounds,
    /// WordMismatch occurs if the puzzle state is incoherent at runtime.
    #[error("puzzle doesn't match list of words")]
    WordMismatch,
}

// !!! User Configuration !!!

/// CrosswordReqs is a structure holding requirements the final puzzle must meet including:
/// "max_retries": how many times to attempt to make a valid puzzle before erroring out,
/// "min_letters_per_word" how small can a word be (if > 3, this value overwrites MIN_LETTER_COUNT)
/// "min_words" the minimum number of words that make up the answer
/// "max_empty_columns" the number of columns that can be completely empty
/// "max_empty_rows" the number of rows that can be completely empty
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct CrosswordReqs {
    /// how many times to attempt to make a valid puzzle before erroring out,
    pub max_retries: usize, // NOTE: This is the only required one
    /// how small can a word be (if > 3, this value overwrites MIN_LETTER_COUNT)
    pub min_letters_per_word: Option<usize>,
    /// the minimum number of words that make up the answer
    pub min_words: Option<usize>,
    /// the number of columns that can be completely empty
    pub max_empty_columns: Option<usize>,
    /// the number of rows that can be completely empty
    pub max_empty_rows: Option<usize>,
}

/// CrosswordInitialPlacementStrategy allows the caller to specifiy how to place the first word
/// of the puzzle. Can be randomly generated using local RNG, which is how default is impl'd.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum CrosswordInitialPlacementStrategy {
    /// Center describes placing the initial word in the vericle and horizontal center of the grid.
    Center(Direction),
    /// Custom requires a placement describing where to place the word and how to orient it.
    Custom(Placement),
    /// LowerLeft places the original word at the lower left corner of the grid.
    LowerLeft(Direction),
    /// LowerRight places the original word at the lower right corner of the grid.
    LowerRight(Direction),
    /// UpperLeft places the original word at the upper left corner of the grid.
    UpperLeft(Direction),
    /// UpperRight places the original word at the upper right corner of the grid.
    UpperRight(Direction),
}

impl CrosswordInitialPlacementStrategy {
    /// A helper function to quickly access a given strategy's direction
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

/// CrosswordInitialPlacement allows the caller to specify where and how large the initial word
/// placed should be.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct CrosswordInitialPlacement {
    /// This differs from CrosswordReq's min_letters_per_word by only applying to the initial word
    pub min_letter_count: Option<usize>,
    /// This determines where the initial word will be placed.
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

/// SolutionConf is the structure used to configure the generation crossword solutions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct SolutionConf {
    /// The possible words to use to construct the puzzle
    pub words: Vec<Word>,
    /// Maximum words used as answers for the generated puzzle
    pub max_words: usize,
    /// How wide the puzzle should be in single letter spaces
    pub width: usize,
    /// How tall the puzzle should be in single letter spaces
    pub height: usize,
    /// Optional requirements for the puzzle, if a generated puzzle does not meet these
    /// then a retry is initiated, the CrosswordReqs structure requires a max_retries field
    /// be specified to avoid retrying endlessly
    pub requirements: Option<CrosswordReqs>,
    /// Optional requirements for the initial placement, allowing a caller to specify where and
    /// how large the inital word placed should be.
    pub initial_placement: Option<CrosswordInitialPlacement>,
}

// !!! Solution Generation and Stateful Puzzles !!!

/// Solution represents a complete crossword structure. Does not include stateful
/// constructs for user input.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Solution {
    /// The actual structure of the crossword, represented as a Vector of Rows. This causes the
    /// traversal logic to use 0,0 as the upper-left corner, for better or worse.
    pub grid: Vec<SolutionRow>,
    /// A vector of words containing all (and only) the solutions to the puzzle.
    pub words: Vec<PlacedWord>,
    /// The width of the puzzle, this matches a SolutionRow.row.len()'s value.
    width: usize,
    /// The height of the puzzle, this matches grid.len()'s value.
    height: usize,
}

impl Solution {
    /// Using the passed in SolutionConf, it attempts to generate a puzzle.
    /// If the puzzle errors out or does not meet specs, generation is retried.
    /// If max_retries are hit (or in the case of no max_retries property, after the first)
    /// Then a CrosswordError::MaxRetries is returned
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

    /// Useful for testing deserialized Solutions from external sources.
    /// Also used as a sanity check in the new function
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
                    if mlc >= MIN_LETTER_COUNT {
                        mlc
                    } else {
                        MIN_LETTER_COUNT
                    }
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
        let mut done = false;
        while !done {
            match words.pop() {
                Some(word) => match self.try_initial_placement(min_letter_count, &strategy, word) {
                    Ok(_) => {
                        break;
                    }

                    Err(w) => {
                        skipped_words.push(w);
                    }
                },
                None => done = true,
            }
        }

        skipped_words.reverse();
        words.append(&mut skipped_words);
        Ok(words)
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

        if count < min_letter_count || count > max_letter_count {
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

/// PuzzleSpace functions as a stateful structure representing whether a char has been guessed
/// for this space. Contains a boolean representing whether it can take a char (true) or is an empty
/// space (false), and has a char_slot that is an `Option<char>`. This could be represented as an
/// `Option<Option<char>>` if this were only targeting Rust, but that approach becomes ambiguous when
/// De/Serializing from JS.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PuzzleSpace {
    char_slot: Option<char>,
    has_char_slot: bool,
}

/// PuzzleRow contains a vector of PuzzleSpaces at "row", represetning a stateful row of the
/// crossword's grid.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PuzzleRow {
    /// row contains a vector of puzzle spaces in various states.
    pub row: Vec<PuzzleSpace>,
}

/// Puzzle is a stateful game composed of a static solution, a stateful grid, and a list of player
/// submitted answers.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct Puzzle {
    /// solution is the underlying solution to the puzzle and source for how to construct the grid
    pub solution: Solution,
    /// player_answers is a vector of player guesses used to display on the grid
    pub player_answers: Vec<PlacedWord>,
    /// grid is a data structure representing the state of crossword of the course of play, used by
    /// front-ends to render the puzzle to the player.
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

    /// Rebuilds the grid from the provided player answers, used to change the state of the Puzzle
    /// struct so that a downstream application can re-render it.
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

    /// Builds the stateful Puzzle struct from a SoultionConf
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

    /// place_answer checks if the word fits, adds the word to the player_answers and grid fields.
    /// If there is a conflict or the word is the wrong length returns an Ok(None).
    /// Returns error if guess is invalid, meaning the placement isn't in the solutions vec
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

    /// remove_answer removes the answer from the Puzzle, then rebuilds the grid.
    pub fn remove_answer(&mut self, placement: &Placement) -> Result<(), CrosswordError> {
        self.player_answers.retain(|w| &w.placement != placement);
        self.grid_from_answers()?;
        Ok(())
    }

    /// is_complete checks if the puzzle's player-supplied answers all match the puzzle's solution.
    /// returns an error if there are more player-supplied answers than words in the solution.
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

    /// wrong_answers_and_solutions return all user answers that are incorrect, along with the
    /// coresponding correction.
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

/// GuessResult encompasses the possible outcomes of a player guess.
/// Not all results are used with all Playmodes.
/// This allows a calling application to distiguish between a bad answer from a player and a bad state
/// in the Puzzle by returning an error in the case of the latter.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub enum GuessResult {
    /// The guess has a valid placement, but the word would overwrite existing answers
    /// Or the word guessed has the wrong length for the placement.
    Conflict,
    /// The guess is correct and completes the puzzle, not returned in the classic puzzle playmode
    Complete,
    /// The guess is correct, not returned in the classic puzzle playmode
    Correct,
    /// The guess is of a word already included in the puzzle.
    Repeat,
    /// The guess fits the placement, but otherwise is unchecked, the primary response of the
    /// classic puzzle playmode
    Unchecked,
    /// The guess is valid but wrong.
    Wrong,
}

/// Playmode is an abstraction over the act of the player guessing a word, depending on the playmode
/// different checks and results will occur.
pub trait Playmode
where
    Self: Sized,
{
    /// guess_word handles a player guess, returns a GuessResult in most cases, and only returns an
    /// error if the Puzzle is in a bad state or the guess' placement is invalid.
    fn guess_word(&mut self, word: PlacedWord) -> Result<GuessResult, CrosswordError>;
}

/// ClassicPuzzle Playmode checks that guesses fit the Placement and don't overwrite existing guesses.
/// It accumulates player answers and can return if it's complete or not, but does not do so
/// automatically on guess like other Playmodes. It exposes a "remove_answer" function to allow the
/// player to remove guesses they deem as bad when dealing with an incorrect Puzzle.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct ClassicPuzzle {
    /// The underlying generic stateful Puzzle struct.
    pub puzzle: Puzzle,
}

impl ClassicPuzzle {
    /// Generates a ClassicPuzzle from a SolutionConf
    pub fn new(conf: SolutionConf) -> Result<ClassicPuzzle, CrosswordError> {
        Ok(ClassicPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }

    /// Remove the answer at the given placement.
    pub fn remove_answer(&mut self, placement: &Placement) -> Result<(), CrosswordError> {
        self.puzzle.remove_answer(placement)
    }
}

impl Playmode for ClassicPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> Result<GuessResult, CrosswordError> {
        // Is this a valid guess?
        if !self
            .puzzle
            .solution
            .words
            .iter()
            .any(|w| w.placement == word.placement)
        {
            return Err(CrosswordError::InvalidPlayerGuess);
        }

        // Remove previous guess at placement if exists
        self.puzzle
            .player_answers
            .retain(|w| w.placement != word.placement);

        // Remove the previous guess from the grid
        self.puzzle.grid_from_answers()?;

        // If this returns None, it means the word had a valid placement but it would
        // overwrite a pre-existing answer's letters or is too short / long for the placement
        match self.puzzle.place_answer(word)? {
            Some(_) => Ok(GuessResult::Unchecked),
            None => Ok(GuessResult::Conflict),
        }
    }
}

/// PlacedWordPuzzle Playmode expects the guess to a placement and word, then only adds the answer
/// to the puzzle state if the guess is valid. Returns a "Complete" result when the last correct guess
/// is given.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PlacedWordPuzzle {
    /// The underlying generic stateful Puzzle struct.
    pub puzzle: Puzzle,
}

impl PlacedWordPuzzle {
    /// Generates a PlacedWordPuzzle from a SolutionConf
    pub fn new(conf: SolutionConf) -> Result<PlacedWordPuzzle, CrosswordError> {
        Ok(PlacedWordPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }
}

impl Playmode for PlacedWordPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> Result<GuessResult, CrosswordError> {
        if self.puzzle.is_complete()? {
            return Ok(GuessResult::Complete);
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
            return Err(CrosswordError::InvalidPlayerGuess);
        };

        if correct == word {
            match self.puzzle.place_answer(word)? {
                Some(_) => {
                    if self.puzzle.is_complete()? {
                        Ok(GuessResult::Complete)
                    } else {
                        Ok(GuessResult::Correct)
                    }
                }
                None => Ok(GuessResult::Conflict),
            }
        } else {
            Ok(GuessResult::Wrong)
        }
    }
}

/// PerWordPuzzle Playmode expects the user to guess a word without a Placement. If the word is
/// present, it is added to the Puzzle at the correct placement. On the last word being correctly
/// guessed, it returns GuessResult::Complete.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[cfg_attr(
    any(doc, target_arch = "wasm32"),
    derive(Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PerWordPuzzle {
    /// The underlying generic stateful Puzzle struct.
    pub puzzle: Puzzle,
}

impl PerWordPuzzle {
    /// Generates a PerWordPuzzle from a SolutionConf
    pub fn new(conf: SolutionConf) -> Result<PerWordPuzzle, CrosswordError> {
        Ok(PerWordPuzzle {
            puzzle: Puzzle::new(conf)?,
        })
    }
}

impl Playmode for PerWordPuzzle {
    fn guess_word(&mut self, word: PlacedWord) -> Result<GuessResult, CrosswordError> {
        if self.puzzle.is_complete()? {
            return Ok(GuessResult::Complete);
        }

        if self
            .puzzle
            .player_answers
            .iter()
            .any(|w| w.word.text == word.word.text)
        {
            Ok(GuessResult::Repeat)
        } else if let Some(c) = self
            .puzzle
            .solution
            .words
            .iter()
            .find(|w| w.word.text == word.word.text)
        {
            match self.puzzle.place_answer(c.clone())? {
                Some(_) => {
                    if self.puzzle.is_complete()? {
                        Ok(GuessResult::Complete)
                    } else {
                        Ok(GuessResult::Correct)
                    }
                }
                None => Ok(GuessResult::Conflict),
            }
        } else {
            Ok(GuessResult::Wrong)
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

// This allows the CrosswordError to be returned from WASM-compiled functions
#[cfg(any(doc, target_arch = "wasm32"))]
#[allow(clippy::from_over_into)]
impl std::convert::Into<JsValue> for CrosswordError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

/// new_solution is the only way JS/WASM applications can construct Solution structs
#[cfg(any(doc, target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn new_solution(conf: SolutionConf) -> Result<Solution, CrosswordError> {
    Solution::new(conf)
}

/// PuzzleType allows both sides of the JS/WASM divide to reference different types of Playmode.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PuzzleType {
    /// Classic coresponds to the ClassicPuzzle struct
    Classic,
    /// PlacedWord coresponds to the PlacedWord struct
    PlacedWord,
    /// PerWord coresponds to the PerWord struct
    PerWord,
}

/// PuzzleContainer allows both sides of the JS/WASM divide to understand the PuzzleType when the
/// struct is passed over the barrier and back. In normal Rust, each of the Playmode structs are
/// identifiable by their type, but when they are De/Serialized, it is impossible to distinguish
/// between them, since they are all of the same shape: { puzzle: Puzzle }.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleContainer {
    /// Acts as a label for "puzzle".
    puzzle_type: PuzzleType,
    /// The underlying stateful Puzzle.
    puzzle: Puzzle,
}

/// new_puzzle is the only way JS/WASM applications can construct Puzzle structs.
/// Requires a PuzzleType which will determine the Puzzle's Playmode and act as the label of the
/// returned Puzzle struct.
#[cfg(any(doc, target_arch = "wasm32"))]
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

/// PuzzleCompleteContainer is used to pass back a puzzle after checking for completeness.
/// This is to allow the JS client to surrender ownership of the puzzle, then have it returned by
/// the WASM function is_puzzle_complete.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleCompleteContainer {
    /// puzzle_container holds the new version of the PuzzleContainer passed in from the client.
    pub puzzle_container: PuzzleContainer,
    /// is_complete reports on whether the puzzle in puzzle_container is complete.
    pub is_complete: bool,
}

/// Takes a PuzzleContainer and returns a PuzzleCompleteContainer with a bool at "is_complete"
/// describing puzzle state. The use of these wrapper containers is to get around ownership issues
/// over the JS/WASM divide. JS passes ownership of the PuzzleContainer to WASM and WASM returns the
/// given PuzzleContainer inside the PuzzleCompleteContainer back to the JS caller.
#[cfg(any(doc, target_arch = "wasm32"))]
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

/// WrongAnswerPair is used to get around the inability to use tuples in WASM by converting a tuple of
/// (got, wanted) to a struct with those labeled fields.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WrongAnswerPair {
    /// got is what the crossword is recording as the player's incorrect guess.
    pub got: PlacedWord,
    /// wanted is what the crossword solution states is the correct guess.
    pub wanted: PlacedWord,
}

/// WrongAnswersContainer is a wrapper used to deal with ownership issues between the JS/WASM divide.
/// When the JS client calls wrong_answers_and_solutions, it passes ownership of the PuzzleContainer
/// that it wants the wrong answers of to WASM. WASM performs the operation, and returns the given
/// PuzzleContainer in this wrapper (along with the requested data) back to the caller.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WrongAnswersContainer {
    /// puzzle_container holds the new version of the PuzzleContainer passed in from the client.
    pub puzzle_container: PuzzleContainer,
    /// wrong_answer_pairs includes the player's incorrect guess next to the right answer for the
    /// given Placement.
    pub wrong_answer_pairs: Vec<WrongAnswerPair>,
}

/// wrong_answers_and_solutions acts as calling puzzle_container.puzzle.wrong_answers_and_solutions()?
/// but formats the output in structs rather than tuples for the calling JS application and returns
/// ownership of the passed-in PuzzleContainer to the JS side.
#[cfg(any(doc, target_arch = "wasm32"))]
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

/// PuzzleAndResult is a wrapper type used to deal with ownership issues between the JS/WASM divide.
/// When a JS application calls guess_word, it surrenders ownership of the PuzzleContainer over to
/// the WASM side, which then performs the requested operation. The WASM then uses this wrapper to
/// return the result of the operation along with ownership of the given PuzzleContainer back to
/// the JS side.
#[cfg(any(doc, target_arch = "wasm32"))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PuzzleAndResult {
    /// puzzle_container holds the new version of the PuzzleContainer passed in from the client.
    puzzle_container: PuzzleContainer,
    /// guess_result contains the result of the guess passed in from the client.
    guess_result: GuessResult,
}

/// guess_word is similar to the native Rust's PlayMode.guess_word(guess) but uses the
/// PuzzleAndResult wrapper type to return ownership of the passed in PuzzleContainer to the JS side.
#[cfg(any(doc, target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn guess_word(
    puzzle_container: PuzzleContainer,
    guess: PlacedWord,
) -> Result<PuzzleAndResult, CrosswordError> {
    let puzzle_type = puzzle_container.puzzle_type;
    let (puzzle, guess_result) = match puzzle_type {
        PuzzleType::Classic => {
            let mut classic = ClassicPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = classic.guess_word(guess)?;
            (classic.puzzle, res)
        }
        PuzzleType::PlacedWord => {
            let mut placed_word = PlacedWordPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = placed_word.guess_word(guess)?;
            (placed_word.puzzle, res)
        }
        PuzzleType::PerWord => {
            let mut per_word = PerWordPuzzle {
                puzzle: puzzle_container.puzzle,
            };

            // NOTE: because guess_word is a mutation, we do it before returning the var
            let res = per_word.guess_word(guess)?;
            (per_word.puzzle, res)
        }
    };

    Ok(PuzzleAndResult {
        puzzle_container: PuzzleContainer {
            puzzle_type,
            puzzle,
        },
        guess_result,
    })
}

/// remove_answer calls puzzle_container.puzzle.remove_answer(&placement), then returns ownership
/// of the PuzzleContainer back to the calling JS side.
#[cfg(any(doc, target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn remove_answer(
    puzzle_container: PuzzleContainer,
    placement: Placement,
) -> Result<PuzzleContainer, CrosswordError> {
    if puzzle_container.puzzle_type != PuzzleType::Classic {
        return Err(CrosswordError::BadPuzzleType);
    }
    let mut puzzle_container = puzzle_container;
    puzzle_container.puzzle.remove_answer(&placement)?;
    Ok(puzzle_container)
}

/// set_panic_hook is a debug feature that is called from npm_pkg/src/crossword_generator_wrapper.ts
/// It improves the quality of error messages that are printed to the dev console
/// For more details see `<https://github.com/rustwasm/console_error_panic_hook#readme>`
#[cfg(any(doc, target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
