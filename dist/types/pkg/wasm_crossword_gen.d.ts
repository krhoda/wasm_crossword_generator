/* tslint:disable */
/* eslint-disable */
/**
* @param {CrosswordConf} conf
* @returns {Crossword}
*/
export function new_crossword(conf: CrosswordConf): Crossword;
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

export interface CrosswordRow {
    row: (string | null)[];
}

export interface Crossword {
    puzzle: CrosswordRow[];
    words: PlacedWord[];
    width: number;
    height: number;
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

export interface CrosswordConf {
    words: Word[];
    max_words: number;
    width: number;
    height: number;
    requirements: CrosswordReqs | null;
    initial_placement: CrosswordInitialPlacement | null;
}


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly new_crossword: (a: number, b: number) => void;
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
