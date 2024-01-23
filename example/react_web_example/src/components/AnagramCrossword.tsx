import React, { useEffect, useState } from 'react';
import Crossword from './Crossword';
import { solutions } from '../data/solutions';
import type { CrosswordClient, PuzzleContainer, SolutionConf } from "wasm_crossword_generator";

export interface AnagramCrosswordProps {
	getClient: () => Promise<CrosswordClient>,
}

const solutionKeys = Object.keys(solutions);

function shuffleString(s: string): string {
    let a = s.split(""),
        n = a.length;

    for(let i = n - 1; i > 0; i--) {
        let j = Math.floor(Math.random() * (i + 1));
        let tmp = a[i];
        a[i] = a[j];
        a[j] = tmp;
    }
    return a.join("");
}

const initPuzzleContainer: PuzzleContainer | null = null;
export default function AnagramCrossword({ getClient }: AnagramCrosswordProps) {
	let [puzzleContainer, setPuzzleContainer] = useState(initPuzzleContainer);
	let [solutionChars, setSolutionChars] = useState("");

	async function newPuzzle(): Promise<void> {
		let client = await getClient();
		let puzzle: PuzzleContainer | null = null;
		let nextSolutionChars = "";

		while (!puzzle) {
			let randomSolutionIndex = Math.floor(Math.random() * solutionKeys.length);
			nextSolutionChars = solutionKeys[randomSolutionIndex];
			let words = solutions[nextSolutionChars];
			let conf: SolutionConf = {
				height: 10,
				width: 10,
				max_words: 20,
				initial_placement: {
					min_letter_count: 6,
					strategy: null,
				},
				words,
				requirements: {
					max_retries: 100,
					min_words: 10,
					min_letters_per_word: 3,
					max_empty_columns: 0,
					max_empty_rows: 0
				}
			};

			try {
				puzzle = client.generate_crossword_puzzle(conf, "PerWord");
			} catch(e) {
				console.error(e);
				puzzle = null;
			}
		}

		setPuzzleContainer(puzzle);
		setSolutionChars(shuffleString(nextSolutionChars));
	}

	useEffect(() => {
		newPuzzle()
	}, [])

	return (
		<Crossword puzzleContainer={puzzleContainer} />
	)
}
