/* tslint:disable */
/* eslint-disable */
/**
* @param {SolutionConf} conf
* @returns {Solution}
*/
export function new_solution(conf: SolutionConf): Solution;
/**
* @param {SolutionConf} conf
* @param {PuzzleType} puzzle_type
* @returns {PuzzleContainer}
*/
export function new_puzzle(conf: SolutionConf, puzzle_type: PuzzleType): PuzzleContainer;
/**
* @param {PuzzleContainer} puzzle_container
* @returns {PuzzleCompleteContainer}
*/
export function is_puzzle_complete(puzzle_container: PuzzleContainer): PuzzleCompleteContainer;
/**
* @param {PuzzleContainer} puzzle_container
* @returns {WrongAnswersContainer}
*/
export function wrong_answers_and_solutions(puzzle_container: PuzzleContainer): WrongAnswersContainer;
/**
* @param {PuzzleContainer} puzzle_container
* @param {PlacedWord} guess
* @returns {PuzzleAndResult}
*/
export function guess_word(puzzle_container: PuzzleContainer, guess: PlacedWord): PuzzleAndResult;
/**
*/
export function set_panic_hook(): void;
export interface Word {
    text: string;
    clue: string | null;
}

export type Direction = "Horizontal" | "Verticle";

export interface Placement {
    x: number;
    y: number;
    direction: Direction;
}

export interface PlacedWord {
    placement: Placement;
    word: Word;
}

export interface SolutionRow {
    row: (string | null)[];
}

export interface CrosswordReqs {
    max_retries: number;
    min_letters_per_word: number | null;
    min_words: number | null;
    max_empty_columns: number | null;
    max_empty_rows: number | null;
}

export type CrosswordInitialPlacementStrategy = { Center: Direction } | { Custom: Placement } | { LowerLeft: Direction } | { LowerRight: Direction } | { UpperLeft: Direction } | { UpperRight: Direction };

export interface CrosswordInitialPlacement {
    min_letter_count: number | null;
    strategy: CrosswordInitialPlacementStrategy | null;
}

export interface SolutionConf {
    words: Word[];
    max_words: number;
    width: number;
    height: number;
    requirements: CrosswordReqs | null;
    initial_placement: CrosswordInitialPlacement | null;
}

export interface Solution {
    grid: SolutionRow[];
    words: PlacedWord[];
    width: number;
    height: number;
}

export interface PuzzleSpace {
    char_slot: string | null;
    has_char_slot: boolean;
}

export interface PuzzleRow {
    row: PuzzleSpace[];
}

export interface Puzzle {
    solution: Solution;
    player_answers: PlacedWord[];
    grid: PuzzleRow[];
}

export type GuessResult = "Complete" | "Correct" | "InvalidPlacement" | "InvalidTooManyAnswers" | "Repeat" | "Unchecked" | "Wrong";

export interface ClassicPuzzle {
    puzzle: Puzzle;
}

export interface PlacedWordPuzzle {
    puzzle: Puzzle;
}

export interface PerWordPuzzle {
    puzzle: Puzzle;
}

export type PuzzleType = "Classic" | "PlacedWord" | "PerWord";

export interface PuzzleContainer {
    puzzle_type: PuzzleType;
    puzzle: Puzzle;
}

export interface PuzzleCompleteContainer {
    puzzle_container: PuzzleContainer;
    is_complete: boolean;
}

export interface WrongAnswerPair {
    got: PlacedWord;
    wanted: PlacedWord;
}

export interface WrongAnswersContainer {
    puzzle_container: PuzzleContainer;
    wrong_answer_pairs: WrongAnswerPair[];
}

export interface PuzzleAndResult {
    puzzle_container: PuzzleContainer;
    guess_result: GuessResult;
}


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly new_solution: (a: number, b: number) => void;
  readonly new_puzzle: (a: number, b: number, c: number) => void;
  readonly is_puzzle_complete: (a: number, b: number) => void;
  readonly wrong_answers_and_solutions: (a: number, b: number) => void;
  readonly guess_word: (a: number, b: number) => number;
  readonly set_panic_hook: () => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
