# WASM Crossword Generator
WASM Crossword Generator is a TypeScript library for the generation and operation of crossword puzzles. The main functionality is written in Rust, which is compiled to WASM with TypeScript types generated from the Rust types. The end result is completely portable and well-typed.


The underlying Rust code is found in the `crate` directory, it's [published on it's own to crates.io](https://crates.io/crates/wasm_crossword_generator) and [documentation is available](https://docs.rs/wasm_crossword_generator/latest/wasm_crossword_generator/).

This directory houses the `src` for the NPM package `wasm_crossword_generator` as well as `example`s in the respective directories.

To add it to your NPM project.
```
$ npm install --save wasm_crossword_generator
```

The most basic usage of the library would look like:
```typescript
import {CrosswordClient, SolutionConf} from "wasm_crossword_generator";
const client = await CrosswordClient.initialize();

const words = [{text: "library", clue: "Not a framework"}];
const conf: SolutionConf = {
  height: 10,
  width: 10,
  max_words: 20,
  initial_placement: null,
  words,
  // A real conf would almost certainly want to pass an options object here to enable retries,
  // but because we only have one word and no acceptance criteria, we know that the puzzle will be
  // created first try.
  requirements: null,
}

// The "PerWord" puzzle only requires players to guess a word, not a word and a placement. It also
// immediately informs the user if the guess was correct.
let puzzle_container = client.generate_crossword_puzzle(conf, "PerWord");

let guess = {
  word: {
    text: "library",
    // The clue is not checked as part of a guess for any Playmode.
    clue: null,
  },
  // The placement is ignored because it is a "PerWord" puzzle, but with other Playmodes
  // this would be meaningful.
  placement { x: 0, y: 0, direction: "Horizontal" },
};

// Note the need to re-assign to puzzle_container. This technique is used to pass ownership
// back and forth with WASM.
let guess_result = null;
{puzzle_container, guess_result} = client.guess_word(puzzle, guess);

// Because there is only one word in the "puzzle", the puzzle is "Complete" after one guess.
assert(guess_result == "Complete")
```

A concrete use of this library is found in the [example Single-Page Application](example/react_web_example).

The `CrosswordClient` abstraction found [here](src/crossword_generator_wrapper.ts) contains most of the information to run the puzzle. The types defined [here](dist/types/pkg/wasm_crossword_generator.d.ts) are useful too.

The playmodes are "Classic", "PlacedWord", and "PerWord". "Classic" doesn't tell the user if the guess is correct or not and allows the user to save (and later, remove) incorrect answers. "PlacedWord" does tell the user if the guess is correct at time of the guess. "PerWord" is like placed word, but does not check the "Placement" portion of a guess.

There are more useful functions attached to the client, like `wrong_answers_and_solutions` and `remove_answer`. Full documentation forthcoming.

Happy puzzling!
