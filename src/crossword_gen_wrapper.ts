import init, {InitInput} from "./pkg/wasm_crossword_gen.js";

// There is some weirdness around re-exporting types using rollup, see:
// https://github.com/rollup/plugins/issues/71
// This was the cleanest way to re-export a type that I have found:
export type Word = import("./pkg/wasm_crossword_gen.js").Word;
export type Direction = import("./pkg/wasm_crossword_gen.js").Direction;
export type PlacedWord = import("./pkg/wasm_crossword_gen.js").PlacedWord;
export type Space = import("./pkg/wasm_crossword_gen.js").Space;
export type CrosswordRow = import("./pkg/wasm_crossword_gen.js").CrosswordRow;
export type Placement = import("./pkg/wasm_crossword_gen.js").Placement;
export type Crossword = import("./pkg/wasm_crossword_gen.js").Crossword;
export type CrosswordConf = import("./pkg/wasm_crossword_gen.js").CrosswordConf;



export type LoadOpts =  {
	wasm?: InitInput
};

let wasmInit: (() => InitInput) | undefined = undefined;
export const setWasmInit = (arg: () => InitInput) => {
  wasmInit = arg;
};

let initialized: Promise<void> | undefined = undefined;

export class PuzzleClient {
	private constructor() {}

	public static initialize = async (options?: LoadOpts): Promise<Sorter> => {
		if (initialized === undefined) {
			//@ts-ignore
			const loadModule = options?.wasm ?? wasmInit();
			initialized = init(loadModule).then(() => void 0);
		}

		await initialized;
		return new PuzzleClient();
	}

	public generate_crossword_puzzle = (a: Array<string>): void => {

	}
}
