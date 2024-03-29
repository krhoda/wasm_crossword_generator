import { InitInput } from "./pkg/wasm_crossword_generator.js";
export type Direction = import("./pkg/wasm_crossword_generator.js").Direction;
export type PlacedWord = import("./pkg/wasm_crossword_generator.js").PlacedWord;
export type Placement = import("./pkg/wasm_crossword_generator.js").Placement;
export type PuzzleAndResult = import("./pkg/wasm_crossword_generator.js").PuzzleAndResult;
export type PuzzleCompleteContainer = import("./pkg/wasm_crossword_generator.js").PuzzleCompleteContainer;
export type PuzzleContainer = import("./pkg/wasm_crossword_generator.js").PuzzleContainer;
export type PuzzleRow = import("./pkg/wasm_crossword_generator.js").PuzzleRow;
export type PuzzleType = import("./pkg/wasm_crossword_generator.js").PuzzleType;
export type Solution = import("./pkg/wasm_crossword_generator.js").Solution;
export type SolutionConf = import("./pkg/wasm_crossword_generator.js").SolutionConf;
export type SolutionRow = import("./pkg/wasm_crossword_generator.js").SolutionRow;
export type Word = import("./pkg/wasm_crossword_generator.js").Word;
export type WrongAnswerPair = import("./pkg/wasm_crossword_generator.js").WrongAnswerPair;
export type WrongAnswersContainer = import("./pkg/wasm_crossword_generator.js").WrongAnswersContainer;
export type LoadOpts = {
    wasm?: InitInput;
};
export declare const setWasmInit: (arg: () => InitInput) => void;
export declare class CrosswordClient {
    private constructor();
    static initialize: (options?: LoadOpts) => Promise<CrosswordClient>;
    generate_crossword_solution: (conf: SolutionConf) => Solution;
    generate_crossword_puzzle: (conf: SolutionConf, puzzle_type: PuzzleType) => PuzzleContainer;
    is_puzzle_complete: (puzzle_container: PuzzleContainer) => PuzzleCompleteContainer;
    wrong_answers_and_solutions: (puzzle_container: PuzzleContainer) => WrongAnswersContainer;
    guess_word: (puzzle_container: PuzzleContainer, guess: PlacedWord) => PuzzleAndResult;
    remove_answer: (puzzle_container: PuzzleContainer, placement: Placement) => PuzzleContainer;
}
