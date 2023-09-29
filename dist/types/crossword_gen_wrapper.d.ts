import { InitInput } from "./pkg/wasm_crossword_gen.js";
export type Word = import("./pkg/wasm_crossword_gen.js").Word;
export type Direction = import("./pkg/wasm_crossword_gen.js").Direction;
export type PlacedWord = import("./pkg/wasm_crossword_gen.js").PlacedWord;
export type CrosswordRow = import("./pkg/wasm_crossword_gen.js").CrosswordRow;
export type Placement = import("./pkg/wasm_crossword_gen.js").Placement;
export type CrosswordConf = import("./pkg/wasm_crossword_gen.js").CrosswordConf;
export type Crossword = import("./pkg/wasm_crossword_gen.js").Crossword;
export type LoadOpts = {
    wasm?: InitInput;
};
export declare const setWasmInit: (arg: () => InitInput) => void;
export declare class CrosswordClient {
    private constructor();
    static initialize: (options?: LoadOpts) => Promise<CrosswordClient>;
    generate_crossword_puzzle: (conf: CrosswordConf) => Crossword;
}
