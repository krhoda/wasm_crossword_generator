/* tslint:disable */
/* eslint-disable */
/**
* @param {string} vec
* @returns {any}
*/
export function quicksort(vec: string): any;
export interface Word {
    text: string;
    clue: string | null;
}

export type Direction = "Horizontal" | "Verticle";

export interface PlacedWord {
    direction: Direction;
    word: Word;
}

export interface Space {
    letter: string | null | null;
}

export interface CrosswordRow {
    row: Space[];
}

export interface Placement {
    x: number;
    y: number;
    direction: Direction;
}

export interface Crossword {
    puzzle: CrosswordRow<W>[];
    words: Word[];
}

export type Sortable = string[] | number[];


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly quicksort: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
