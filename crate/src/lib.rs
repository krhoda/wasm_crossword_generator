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
    pub letter: Option<Option<char>>,
}

// NOTE: This is used to avoid impling Copy on structs with strings.
const DEFAULT_SPACE: Space = Space { letter: Some(None) };

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
    pub words: Vec<Word>,
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

impl<const W: usize, const H: usize> Crossword<W, H> {
    pub fn new(words: Vec<Word>, max_words: usize) -> Crossword<W, H> {
        let mut crossword = Crossword::<W, H>::new_empty();
        let mut words = words;
        words.shuffle(&mut thread_rng());

        for word in words {
            // If max words hit, break.
            if crossword.words.len() >= max_words {
                break;
            }

            if crossword.words.is_empty() {
                let _ = crossword.place(word, 0, 0);
                continue;
            }

            let count_at_current_word = crossword.words.len();
            for letter in word.text.chars() {
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

                        if let Some(Some(c)) = current_puzzle[row_count].row[col_count].letter {
                            if c == letter {
                                // TODO: VERIFY THIS ISN'T BACKWARDS!
                                let _ = crossword.place(word.clone(), col_count, row_count);
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
            words: Vec::<Word>::new(),
        }
    }

    fn is_empty(&self) -> bool {
        for row in self.puzzle.iter() {
            for space in row.row {
                if let Some(Some(_)) = space.letter {
                    return false;
                }
            }
        }

        true
    }

    // place takes a word and a possible intersection of the word.
    // The first word is special cased.
    fn place(&mut self, word: Word, x: usize, y: usize) -> Result<(), CrosswordError> {
        if x > W || y > H {
            return Err(CrosswordError::PointOutOfBounds);
        }

        let row = self.puzzle[y].row;
        let intersection = row[x];

        if self.is_empty() {
            // TODO: Something!
            // let mut direction: Direction = rand::random();
            // if !self.can_place(word, x, y, direction) {
            //     direction = direction.other();
            //     if !self.can_place(word, x, y, direction) {
            //         return Err(CrosswordError::BadFit);
            //     }
            // }

            // let chars = word.text.chars();

            return Ok(());
        }

        if let Some(Some(letter)) = intersection.letter {
            Ok(())
        } else {
            Err(CrosswordError::EmptyIntersection)
        }
    }

    fn can_place(&mut self, word: Word, x: usize, y: usize) -> bool {
        false
    }
}

// Below here is the demo code
// TODO: REMOVE!

// This section is glue between JS and Rust:

// This will be the shared type between JS and Rust.
// This is the type that JSON will be deserailized into on the Rust side.
// It will also generate a Typescript type exposed by the JS libraries.
#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
pub enum Sortable {
    Strings(Vec<String>),
    Numbers(Vec<f64>),
}

// This will treat the destination type (Sortable) generically.
fn quicksort_interface(s: Sortable) -> Sortable {
    match s {
        Sortable::Strings(mut v) => {
            qsort(&mut v);
            Sortable::Strings(v)
        }
        Sortable::Numbers(mut v) => {
            qsort(&mut v);
            Sortable::Numbers(v)
        }
    }
}

// Other than Sortable's type definition, this is the only thing exposed
// to the consuming libraries.
// The lack of typing at the argument level is hidden behind the better
// typing of the JavaScript wrapper -- see quicksort_wrapper.ts
#[wasm_bindgen]
pub fn quicksort(vec: String) -> Result<JsValue, JsError> {
    let vec: Sortable = from_str(&vec).map_err(|e| JsError::new(&format!("{}", e)))?;
    let res = to_string(&quicksort_interface(vec)).map_err(|e| JsError::new(&format!("{}", e)))?;
    Ok(res.into())
}

// This section is the actual implementation in Rust terms.

// The next three functions are a tiny modification to the implementation found here:
// https://www.hackertouch.com/quick-sort-in-rust.html
fn qsort<T: PartialEq + PartialOrd>(arr: &mut [T]) {
    let len = arr.len();
    _quicksort(arr, 0, (len - 1) as isize);
}

fn _quicksort<T: PartialEq + PartialOrd>(arr: &mut [T], low: isize, high: isize) {
    if low < high {
        let p = partition(arr, low, high);
        _quicksort(arr, low, p - 1);
        _quicksort(arr, p + 1, high);
    }
}

fn partition<T: PartialEq + PartialOrd>(arr: &mut [T], low: isize, high: isize) -> isize {
    let pivot = high as usize;
    let mut store_index = low - 1;
    let mut last_index = high;

    loop {
        store_index += 1;
        while arr[store_index as usize] < arr[pivot] {
            store_index += 1;
        }
        last_index -= 1;
        while last_index >= 0 && arr[last_index as usize] > arr[pivot] {
            last_index -= 1;
        }
        if store_index >= last_index {
            break;
        } else {
            arr.swap(store_index as usize, last_index as usize);
        }
    }
    arr.swap(store_index as usize, pivot);
    store_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qsort() {
        let expected = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];

        let mut unsort1 = vec![
            "B".to_string(),
            "A".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];

        let mut unsort2 = vec![
            "D".to_string(),
            "C".to_string(),
            "B".to_string(),
            "A".to_string(),
        ];

        let mut unsort3 = vec![
            "C".to_string(),
            "B".to_string(),
            "D".to_string(),
            "A".to_string(),
        ];

        // sanity check:
        assert_eq!(&expected, &expected);
        assert_ne!(&expected, &unsort1);
        assert_ne!(&expected, &unsort2);
        assert_ne!(&expected, &unsort3);

        qsort(&mut unsort1);
        qsort(&mut unsort2);
        qsort(&mut unsort3);

        assert_eq!(&expected, &unsort1);
        assert_eq!(&expected, &unsort2);
        assert_eq!(&expected, &unsort3);

        let expected = [1, 2, 3];
        let mut unsort1 = [3, 2, 1];
        let mut unsort2 = [2, 3, 1];

        // sanity check:
        assert_eq!(&expected, &expected);
        assert_ne!(&expected, &unsort1);
        assert_ne!(&expected, &unsort2);

        qsort(&mut unsort1);
        qsort(&mut unsort2);

        assert_eq!(&expected, &unsort1);
        assert_eq!(&expected, &unsort2);
    }
}
