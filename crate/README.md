# WASM Crossword Generator

`wasm_crossword_generator` is a library for generating and operating crossword puzzles with
first-class WebAssembly (WASM) support targeting the browser and other WASM environments. While
fully functional and ergonomic as a Rust library, the design is WASM-first so some trade-offs are
made such as not using const generics, avoiding `Option<Option<T>>` because of ambiguity during
JSON de/serialization, and using structs over tuples.

The most basic example of usage in pure Rust looks like:

```rust
use wasm_crossword_generator::*;

let words: Vec<Word> = vec![
  Word { text: "library".to_string(), clue: Some("Not a framework.".to_string()) },
  // In a real usage, there would be more entries.
];

// A real SolutionConf would probably want "requirements" to allow for retrying crossword
// generation. Because there is only one word, we know we'll get the world's simplest puzzle in
// one try.
let solution_conf = SolutionConf {
    words: words,
    max_words: 20,
    width: 10,
    height: 10,
    requirements: None,
    initial_placement: None,
};

// A PerWordPuzzle is one where the player only guesses words, not locations. The player is
// immediately informed if the guess is correct or not.
let mut puzzle = PerWordPuzzle::new(solution_conf)?;

let guess = PlacedWord {
  // Because it is a PerWordPuzzle, the placement is ignored, unlike other Playmodes.
  placement: Placement { x: 0, y: 0, direction: rand::random() },
  // NOTE: you don't need to match the "clue" field, it is ignored for purposes of PartialEq
  word: Word { text: "library".to_string(), clue: None }
};
let guess_result = puzzle.guess_word(guess)?;

// Because there is only one word, the guess will result in "Complete" instead of "Correct"
assert_eq!(guess_result, GuessResult::Complete);
```

More detail is found in the offical documentation [here](https://docs.rs/wasm_crossword_generator/latest/wasm_crossword_generator/). In the directory above this project, this crate is packaged for use in the brower and published to NPM, more details on that are found there.  In the `../example` folder, runnable, self-contained browser applications demonstrate this library in use.
