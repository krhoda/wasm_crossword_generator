mod utils;

use std::array;

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
    #[error("point out of bounds")]
    PointOutOfBounds,
    #[error("intersection point can only be empty for placement of first word")]
    EmptyIntersection,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CrosswordConf {
    pub words: Vec<Word>,
    pub max_words: usize,
    pub width: usize,
    pub height: usize,
}

#[wasm_bindgen]
pub fn new_crossword(s: &str) -> Crossword {
    let wrapped_conf = serde_json::from_str::<CrosswordConf>(s);
    if wrapped_conf.is_err() {
        return Crossword::new_empty(0, 0);
    }

    let conf = wrapped_conf.unwrap();
    Crossword::new(conf)
}

#[wasm_bindgen]
impl Crossword {
    pub fn new(conf: CrosswordConf) -> Crossword {
        let mut crossword = Crossword::new_empty(conf.width, conf.height);

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

                        if let Some(c) = current_puzzle[row_count].row[col_count] {
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

    fn new_empty(width: usize, height: usize) -> Crossword {
        let mut puzzle = Vec::<CrosswordRow>::new();

        for _ in 0..height {
            puzzle.push(CrosswordRow::new(width))
        }

        Crossword {
            puzzle,
            words: Vec::<PlacedWord>::new(),
            width: 0,
            height: 0,
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
        if x > (self.width - 1) || y > (self.height - 1) {
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
        word_index: usize,
    ) -> Option<Direction> {
        // Sanity check:
        if word_index > word.text.len() - 1 {
            return None;
        }

        let intersection_letter_word = word.text.chars().collect::<Vec<char>>()[word_index];
        if let Some(intersection_letter_puzzle) = self.puzzle[y].row[x] {
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
            self.width
        } else {
            self.height
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

            if let Some(c) = space {
                if letter != c {
                    return false;
                }
            }
        }
        true
    }
}
