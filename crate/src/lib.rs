mod utils;

use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    thread_rng, Rng,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use thiserror::Error;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

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
    pub direction: Direction,
    pub word: Word,
}

#[derive(Copy, Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Space {
    pub letter: Option<char>,
}

// NOTE: This is used to avoid impling Copy on structs with strings.
const DEFAULT_SPACE: Space = Space { letter: None };

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordRow<const W: usize> {
    #[serde(with = "serde_arrays")]
    pub row: [Space; W],
}

impl<const W: usize> CrosswordRow<W> {
    fn new() -> CrosswordRow<W> {
        CrosswordRow::<W> {
            row: [DEFAULT_SPACE; W],
        }
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
pub struct Crossword<const W: usize, const H: usize> {
    #[serde(with = "serde_arrays")]
    pub puzzle: [CrosswordRow<W>; H],
    pub words: Vec<PlacedWord>,
}

#[derive(Error, Debug)]
pub enum CrosswordError {
    #[error("word doesn't fit")]
    BadFit,
    #[error("point out of bounds")]
    PointOutOfBounds,
    #[error("intersection point can only be empty for placement of first word")]
    EmptyIntersection,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordConf {
    words: Vec<Word>,
    max_words: usize,
}

impl<const W: usize, const H: usize> Crossword<W, H> {
    pub fn new(serialized_conf: &str) -> Crossword<W, H> {
        let mut crossword = Crossword::<W, H>::new_empty();

        let wrapped_conf = serde_json::from_str::<CrosswordConf>(serialized_conf);
        if wrapped_conf.is_err() {
            return crossword;
        }

        let conf = wrapped_conf.unwrap();
        let mut words = conf.words;
        let max_words = conf.max_words;

        words.shuffle(&mut thread_rng());

        for word in words {
            // If max words hit, break.
            if crossword.words.len() >= max_words {
                break;
            }

            if crossword.words.is_empty() {
                let _ = crossword.place(word, 0, 0, 0);
                continue;
            }

            let count_at_current_word = crossword.words.len();
            for (word_index, letter) in word.text.chars().enumerate() {
                // TODO: Find a cleaner way to break out?
                if count_at_current_word != crossword.words.len() {
                    break;
                }

                // Clone to avoid borrow issues.
                let current_puzzle = crossword.puzzle.clone();

                for (row_count, _) in current_puzzle.iter().enumerate() {
                    if count_at_current_word != crossword.words.len() {
                        break;
                    }

                    for (col_count, _) in current_puzzle[row_count].row.iter().enumerate() {
                        if count_at_current_word != crossword.words.len() {
                            break;
                        }

                        if let Some(c) = current_puzzle[row_count].row[col_count].letter {
                            if c == letter {
                                // TODO: VERIFY THIS ISN'T BACKWARDS!
                                let _ =
                                    crossword.place(word.clone(), col_count, row_count, word_index);
                            }
                        }
                    }
                }
            }
        }

        crossword
    }

    fn new_empty() -> Crossword<W, H> {
        let puzzle: [CrosswordRow<W>; H] = std::array::from_fn(|_| CrosswordRow::<W>::new());
        Crossword {
            puzzle,
            words: Vec::<PlacedWord>::new(),
        }
    }

    // place takes a word and a possible intersection of the word.
    // The first word is special cased.
    fn place(
        &mut self,
        word: Word,
        x: usize,
        y: usize,
        word_index: usize,
    ) -> Result<(), CrosswordError> {
        if x > (W - 1) || y > (H - 1) {
            return Err(CrosswordError::PointOutOfBounds);
        }

        let placement_direction = self.can_place(&word, x, y, word_index);

        if let Some(direction) = placement_direction {
            let origin = if let Direction::Horizontal = direction {
                x
            } else {
                y
            };

            for (letter_count, letter) in word.text.chars().enumerate() {
                let next_index = if letter_count < word_index {
                    origin - (word_index - letter_count)
                } else if letter_count > word_index {
                    origin + (letter_count - word_index)
                } else {
                    origin
                };

                match direction {
                    Direction::Horizontal => {
                        self.puzzle[y].row[next_index] = Space {
                            letter: Some(letter),
                        };
                    }
                    Direction::Verticle => {
                        self.puzzle[next_index].row[x] = Space {
                            letter: Some(letter),
                        };
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
        word_index: usize,
    ) -> Option<Direction> {
        // Sanity check:
        if word_index > word.text.len() - 1 {
            return None;
        }

        let intersection_letter_word = word.text.chars().collect::<Vec<char>>()[word_index];
        if let Some(intersection_letter_puzzle) = self.puzzle[y].row[x].letter {
            if intersection_letter_puzzle != intersection_letter_word {
                return None;
            }
        };

        let first_try: Direction = rand::random();
        let second_try = first_try.other();

        if self._can_place(word, x, y, word_index, &first_try) {
            Some(first_try)
        } else if self._can_place(word, x, y, word_index, &second_try) {
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
        word_index: usize,
        direction: &Direction,
    ) -> bool {
        let last_index = word.text.len() - 1;
        // Is the word index out of bounds?
        if word_index > last_index {
            return false;
        }

        let remainder = last_index - word_index;

        let origin = if let Direction::Horizontal = direction {
            x
        } else {
            y
        };

        // Would the word go over the top or the left of the puzzle's bounds?
        if word_index > origin {
            return false;
        }

        let edge = if let Direction::Horizontal = direction {
            W
        } else {
            H
        };

        // Would the word go over the bottom or the right of the puzzle's bounds?
        if origin + remainder > edge {
            return false;
        }

        for (letter_count, letter) in word.text.chars().enumerate() {
            let next_index = if letter_count < word_index {
                origin - (word_index - letter_count)
            } else if letter_count > word_index {
                origin + (letter_count - word_index)
            } else {
                origin
            };

            let space = match direction {
                Direction::Horizontal => self.puzzle[y].row[next_index],
                Direction::Verticle => self.puzzle[next_index].row[x],
            };

            if let Some(c) = space.letter {
                if letter != c {
                    return false;
                }
            }
        }
        true
    }
}
