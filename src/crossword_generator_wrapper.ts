import init, {
	guess_word,
	is_puzzle_complete,
	set_panic_hook,
	new_solution,
	new_puzzle,
	remove_answer,
	wrong_answers_and_solutions,
	InitInput
} from "./pkg/wasm_crossword_generator.js";

// There is some weirdness around re-exporting types using rollup, see:
// https://github.com/rollup/plugins/issues/71
// This was the cleanest way to re-export a type that I have found:
export type Direction = import("./pkg/wasm_crossword_generator.js").Direction;
export type PlacedWord = import("./pkg/wasm_crossword_generator.js").PlacedWord;
export type Placement = import("./pkg/wasm_crossword_generator.js").Placement;
export type PuzzleAndResult = import("./pkg/wasm_crossword_generator.js").PuzzleAndResult;
export type PuzzleCompleteContainer = import("./pkg/wasm_crossword_generator.js").PuzzleCompleteContainer;
export type PuzzleContainer = import("./pkg/wasm_crossword_generator.js").PuzzleContainer;
export type PuzzleRow = import("./pkg/wasm_crossword_generator.js").PuzzleRow;
export type PuzzleType = import("./pkg/wasm_crossword_generator.js").PuzzleType;
export type Solution = import("./pkg/wasm_crossword_generator.js").Solution;
export type SolutionConf = import("./pkg/wasm_crossword_generator.js").SolutionConf;
export type SolutionRow = import("./pkg/wasm_crossword_generator.js").SolutionRow;
export type Word = import("./pkg/wasm_crossword_generator.js").Word;
export type WrongAnswerPair = import("./pkg/wasm_crossword_generator.js").WrongAnswerPair;
export type WrongAnswersContainer = import("./pkg/wasm_crossword_generator.js").WrongAnswersContainer;

export type LoadOpts = {
	wasm?: InitInput
};

let wasmInit: (() => InitInput) | undefined = undefined;
export const setWasmInit = (arg: () => InitInput) => {
	wasmInit = arg;
};

let initialized: Promise<void> | undefined = undefined;

export class CrosswordClient {
	private constructor() {}

	public static initialize = async (options?: LoadOpts): Promise<CrosswordClient> => {
		if (initialized === undefined) {
			//@ts-ignore
			const loadModule = options?.wasm ?? wasmInit();
			initialized = init(loadModule).then(() => void 0);
		}

		await initialized;
		set_panic_hook();
		return new CrosswordClient();
	}

	public generate_crossword_solution = (conf: SolutionConf): Solution => {
		return new_solution(conf);
	}

	public generate_crossword_puzzle = (
		conf: SolutionConf,
		puzzle_type: PuzzleType
	): PuzzleContainer => {
		return new_puzzle(conf, puzzle_type);
	}

	public is_puzzle_complete = (
		puzzle_container: PuzzleContainer
	): PuzzleCompleteContainer => {
		return is_puzzle_complete(puzzle_container);
	}

	public wrong_answers_and_solutions = (
		puzzle_container: PuzzleContainer
	): WrongAnswersContainer => {
		return wrong_answers_and_solutions(puzzle_container);
	}

	public guess_word = (
		puzzle_container: PuzzleContainer,
		guess: PlacedWord
	): PuzzleAndResult => {
		return guess_word(puzzle_container, guess);
	}

	public remove_answer = (
		puzzle_container: PuzzleContainer,
		placement: Placement
	): PuzzleContainer => {
		return remove_answer(puzzle_container, placement);
	}
}
