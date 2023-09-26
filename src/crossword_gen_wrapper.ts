import init, {InitInput, quicksort} from "./pkg/wasm_crossword_gen.js";

// There is some weirdness around re-exporting types using rollup, see:
// https://github.com/rollup/plugins/issues/71
// This was the cleanest way to re-export a type that I have found:
export type Sortable = import("./pkg/wasm_crossword_gen.js").Sortable;

export type LoadOpts =  {
	wasm?: InitInput
};

let wasmInit: (() => InitInput) | undefined = undefined;
export const setWasmInit = (arg: () => InitInput) => {
  wasmInit = arg;
};

let initialized: Promise<void> | undefined = undefined;

export class Sorter {
	private constructor() {}

	public static initialize = async (options?: LoadOpts): Promise<Sorter> => {
		if (initialized === undefined) {
			//@ts-ignore
			const loadModule = options?.wasm ?? wasmInit();
			initialized = init(loadModule).then(() => void 0);
		}

		await initialized;
		return new Sorter();
	}

	public sort = (sortable: Sortable): Sortable => {
		let s = JSON.stringify(sortable);
		s = quicksort(s);
		return JSON.parse(s) as Sortable;
	}
}
