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
const initBadGuesses: Array<string> = [];

export default function AnagramCrossword({ getClient }: AnagramCrosswordProps) {
	let [puzzleContainer, setPuzzleContainer] = useState(initPuzzleContainer);
	let [solutionChars, setSolutionChars] = useState("");
	let [showBadGuesses, setShowBadGuesses] = useState(false);
	let [badGuesses, setBadGuesses] = useState(initBadGuesses);
	let [isComplete, setIsComplete] = useState(false);

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
				max_words: 22,
				initial_placement: {
					min_letter_count: nextSolutionChars.length,
					strategy: { Center: "Horizontal" },
				},
				words,
				requirements: {
					max_retries: 100,
					min_words: 15,
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

		setIsComplete(false);
		setPuzzleContainer(puzzle);
		setSolutionChars(shuffleString(nextSolutionChars));
		setBadGuesses([]);
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
		if (selectedLettersContains(c, i)) {
			setSelectedLetters([]);
		} else {
			setSelectedLetters([
				...selectedLetters,
				{ letter: c, index: i }
			]);
		}
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
			let { puzzle_container, guess_result } = client.guess_word(puzzleContainer, {
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

			if (guess_result == "Wrong") {
				alert("Wrong guess!");
				setBadGuesses([...badGuesses, selectedLetters.map((sl) => {
					return sl.letter;
				}).join("")]);
			} else if (guess_result == "Complete") {
				alert("Increadible! You've won!");
				setIsComplete(true);
			} else {
				alert("Amazing!");
			}
			setPuzzleContainer(puzzle_container);
			setSelectedLetters([]);
		}

		// TODO: err out?
	}

	return (
		<Fragment>

			<Crossword puzzleContainer={puzzleContainer} />
			{isComplete ?
				<Fragment>
					<p className="themed-p">Congratulations, you've completed the puzzle!</p>
					<button className="guess-button" onClick={newPuzzle}>
						Generate a new puzzle!
					</button>
				</Fragment>
				:
				<Fragment>
					<p className="themed-p">Selected Letters: {selectedLetters.map((s) => (`${s.letter}`))}</p>
					<button className="guess-button" disabled={selectedLetters.length < 3} onClick={guess}>
						{selectedLetters.length < 3 ? "Enter a Guess!" : "Guess Word?"}
					</button>

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
					<div className="bad-guess-container">
						<p className="themed-p" onClick={() => setShowBadGuesses(!showBadGuesses)}>
							<span className="show-hide" >
								{showBadGuesses ? "Hide" : "Show"} bad guesses
							</span>
						</p>
						{showBadGuesses ?
							(badGuesses.length > 0 ?
								<p className="themed-p">Already Guessed: {badGuesses.join(", ")}</p>
								: <p className="themed-p">No bad guesses yet!</p>)
							: ""}
					</div>
				</Fragment>
			}
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
