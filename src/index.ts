export * from "./index_core.js";
import wasm_crossword_gen_wasm from "./pkg/wasm_crossword_gen_bg.wasm";
import {setWasmInit} from "./crossword_gen_wrapper.js"

// @ts-ignore
setWasmInit(() => wasm_crossword_gen_wasm());
