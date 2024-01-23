import React, { Fragment, useEffect, useState } from 'react';
import Crossword from './Crossword';
import { solutions } from '../data/solutions';
import { CrosswordClient, PuzzleContainer, SolutionConf } from "wasm_crossword_generator";

export interface AnagramCrosswordProps {
	getClient: () => Promise<CrosswordClient>
};

const solutionKeys = Object.keys(solutions);

type SelectedLetter = {
	letter: string,
	index: number,
};

const initialSelectedLetters: Array<SelectedLetter> = [];

function shuffleString(s: string): string {
	let a = s.split(""),
		n = a.length;

	for (let i = n - 1; i > 0; i--) {
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
			} catch (e) {
				console.error(e);
				puzzle = null;
			}
		}

		setPuzzleContainer(puzzle);
		setSolutionChars(shuffleString(nextSolutionChars));
	};

	let [selectedLetters, setSelectedLetters] = useState(initialSelectedLetters);
	useEffect(() => {
		newPuzzle();
		return () => {
			setPuzzleContainer(initPuzzleContainer);
			setSolutionChars("");
			setSelectedLetters(initialSelectedLetters);
		}
	}, []);

	function letterSelectorHandler(c: string, i: number) {
		console.log(selectedLetters, c, i);
		if (selectedLettersContains(c, i)) {
			console.log("In true")
			setSelectedLetters([]);
		} else {
			console.log("In false")
			setSelectedLetters([
				...selectedLetters,
				{ letter: c, index: i }
			]);
		}
		console.log(selectedLetters, c, i);
	}

	function selectedLettersContains(c: string, i: number): boolean {
		for (let j = 0, x = selectedLetters.length; j < x; j++) {
			let sl = selectedLetters[j];
			if (sl.letter == c && sl.index == i) {
				return true;
			}
		}

		return false;
	}

	async function guess() {
		if (puzzleContainer) {
			let client = await getClient();
			let {puzzle_container, guess_result} = client.guess_word(puzzleContainer, {
				placement: {
					x: 0,
					y: 0,
					direction: "Horizontal"
				},
				word: {
					text: selectedLetters.map((sl) => {
						return sl.letter;
					}).join(""),
					clue: null
				}
			});

			console.log(guess_result);
			setPuzzleContainer(puzzle_container);
		}

		// TODO: err out?
	}

	return (
		<Fragment>
			<Crossword puzzleContainer={puzzleContainer} />
			<p>Selected Letters: {selectedLetters.map((s) => (`${s.letter}`))}</p>
			{selectedLetters.length > 2 ? (
				<button onClick={guess}>Guess Word?</button>
			) : ""}
			<div className="letter-container">
				{solutionChars.split("")
					.map(
						(c, i) => {
							return (
								<button
									key={`${c}-${i}`}
									className={
										`letter-button letter-button-${selectedLettersContains(c, i) ? "selected" : "unselected"}`
									}
									onClick={() => { letterSelectorHandler(c, i) }}
								>
									{c}
								</button>
							)
						}
					)}
			</div>
		</Fragment>
	);
};

// <LetterSelector solutionChars={solutionChars} selectedLetters={selectedLetters} setSelectedLetters={setSelectedLetters} />

type LetterSelectorProps = {
	solutionChars: string,
	selectedLetters: Array<SelectedLetter>,
	setSelectedLetters: (nextSelectedLetters: Array<SelectedLetter>) => void,
};

function LetterSelector({ solutionChars, selectedLetters, setSelectedLetters }: LetterSelectorProps) {
	const charArray = solutionChars.split("");
	function f(c: string, i: number): SelectedLetter {
		return { letter: c, index: i }
	};

	return (
		<div className="letter-container">
			{charArray
				.map(f)
				.map(
					(selectedLetter) => (
						<button
							key={`${selectedLetter.letter}-${selectedLetter.index}`}
							className={
								`letter-button letter-button-${selectedLetters.includes(selectedLetter) ? "selected" : "unselected"}`
							}
							onClick={() => {
								console.log(selectedLetter);
								console.log(selectedLetters);
								if (selectedLetters.includes(selectedLetter)) {
									setSelectedLetters([]);
								} else {
									setSelectedLetters([
										...selectedLetters,
										selectedLetter
									]);
								}
							}}

						>
							{selectedLetter.letter}
						</button>
					)
				)
			}
		</div>
	);
}
