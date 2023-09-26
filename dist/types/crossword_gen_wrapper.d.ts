import { InitInput } from "./pkg/wasm_crossword_gen.js";
export type Sortable = import("./pkg/wasm_crossword_gen.js").Sortable;
export type LoadOpts = {
    wasm?: InitInput;
};
export declare const setWasmInit: (arg: () => InitInput) => void;
export declare class Sorter {
    private constructor();
    static initialize: (options?: LoadOpts) => Promise<Sorter>;
    sort: (sortable: Sortable) => Sortable;
}
