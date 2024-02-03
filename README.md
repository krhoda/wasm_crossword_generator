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
  // but because we only have one word and no acceptance criteria,
  // we know that the puzzle will be created first try.
  requirements: null,
}
```

## Inspirations:
https://www.baeldung.com/cs/generate-crossword-puzzle
https://mitchum.blog/building-a-crossword-puzzle-generator-with-javascript/
