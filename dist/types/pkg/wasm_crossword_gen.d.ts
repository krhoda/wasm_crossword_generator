/* tslint:disable */
/* eslint-disable */
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
    letter: string | null;
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
    words: PlacedWord[];
}

export interface CrosswordConf {
    words: Word[];
    max_words: number;
}


export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
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
