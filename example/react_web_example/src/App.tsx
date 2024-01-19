import React from 'react';
import logo from './logo.svg';
import './App.css';
import { CrosswordClient } from "wasm_crossword_gen";
import type { SolutionConf, Solution } from "wasm_crossword_gen";
import { hardcoded_conf } from "./util/crossword_conf";

function App() {
	CrosswordClient.initialize().then((client) => {
		try {
			let puzzle_container = client.generate_crossword_puzzle(
				hardcoded_conf,
				"PlacedWord",
			);
			console.log(puzzle_container);

			for (
				let i = 0, x = puzzle_container.puzzle.solution.words.length;
				i < x;
				i++
			) {
				let word = puzzle_container.puzzle.solution.words[i];
				let res = client.guess_word(puzzle_container, word);
				puzzle_container = res.puzzle_container;
				let guess_result = res.guess_result;

				if (guess_result !== "Correct" && guess_result !== "Complete") {
					console.log("Got Bad Result:", guess_result, word);
				}
			}

			let res = client.is_puzzle_complete(puzzle_container);
			puzzle_container = res.puzzle_container;
			if (res.is_complete) {
				console.log("Success!");
				console.log(puzzle_container);
			} else {
				console.log("Failure!");
			}
		} catch (e) {
			console.error(e);
		}
	});

	return (
		<div className="App">
			<header className="App-header">
				<img src={logo} className="App-logo" alt="logo" />
				<p>
					Edit <code>src/App.tsx</code> and save to reload.
				</p>
				<a
					className="App-link"
					href="https://reactjs.org"
					target="_blank"
					rel="noopener noreferrer"
				>
					Learn React
				</a>
			</header>
		</div>
	);
}

export default App;
