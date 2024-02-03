export * from "./index_core.js";
import wasm_crossword_generator_wasm from "./pkg/wasm_crossword_generator_bg.wasm";
import {setWasmInit} from "./crossword_generator_wrapper.js"

// @ts-ignore
setWasmInit(() => wasm_crossword_generator_wasm());
