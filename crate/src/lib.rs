mod utils;

use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Word {
    pub text: String,
    pub clue: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Direction {
    Horizontal,
    Verticle,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PlacedWord {
    pub direction: Direction,
    pub word: Word,
}

#[derive(Clone, Deserialize, Serialize, Tsify)]
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

impl<const W: usize, const H: usize> Crossword<W, H> {
    fn new() -> Crossword<W, H> {
        let puzzle: [CrosswordRow<W>; H] = std::array::from_fn(|_| CrosswordRow::<W>::new());
        Crossword {
            puzzle,
            words: Vec::<Word>::new(),
        }
    }

    fn place(&self, word: Word, x: usize, y: usize, direction: Direction) {
        // TODO: Something!
    }

    fn can_place(&self, word: Word, x: usize, y: usize) -> Option<Placement> {
        // TODO: Something!
        None
    }
}

pub fn gen_crossword_first_draft<const W: usize, const H: usize>(
    words: Vec<Word>,
    max_words: usize,
) -> Crossword<W, H> {
    let puzzle = Crossword::<W, H>::new();

    let mut words = words;
    words.shuffle(&mut thread_rng());

    let mut word = words.pop();
    if word.is_none() {
        return puzzle;
    }

    puzzle.place(word.unwrap(), 0, 0, Direction::Horizontal);

    let mut word_count: usize = 1;
    word = words.pop();

    while word.is_some() && word_count < max_words {
        let w = word.unwrap();
        let count_at_current_word = word_count;
        for letter in w.clone().text.chars() {
            // TODO: Find a cleaner way to break out?
            if count_at_current_word != word_count {
                break;
            }
            for (row_count, _) in puzzle.puzzle.iter().enumerate() {
                if count_at_current_word != word_count {
                    break;
                }
                for (col_count, _) in puzzle.puzzle[row_count].row.iter().enumerate() {
                    if count_at_current_word != word_count {
                        break;
                    }
                    if let Some(Some(c)) = puzzle.puzzle[row_count].row[col_count].letter {
                        if c == letter {
                            // TODO: VERIFY THIS ISN'T BACKWARDS!
                            match puzzle.can_place(w.clone(), col_count, row_count) {
                                None => {}
                                Some(placement) => {
                                    puzzle.place(
                                        w.clone(),
                                        placement.x,
                                        placement.y,
                                        placement.direction,
                                    );
                                    word_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        word = words.pop();
    }

    puzzle
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
