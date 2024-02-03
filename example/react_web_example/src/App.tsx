import React, { useEffect, useState } from 'react';
import logo from './logo.svg';
import './App.css';
import AnagramCrossword from "./components/AnagramCrossword";
// TODO: Fix issue with PuzzleType import
import { CrosswordClient } from "wasm_crossword_generator";

// Use let-over-lambda to get the power of a global while dealing with one-time async initialization.
function getClientGenerator(): () => Promise<CrosswordClient> {
	let client: CrosswordClient | null = null;
	return async () => {
		if (!client) {
			client = await CrosswordClient.initialize();
		}
		return client
	}
}

const getClient = getClientGenerator();

function App() {
	let [showInstructions, setShowInstructions] = useState(false);

	return (
		<div className="App">
			<header className="App-header">
				<div className="logo_row">
					<img src={logo} className="App-logo" alt="logo" />
					<Plus />
					<div className="wasm_container">
						<WasmLogo />
					</div>
				</div>
				<p className="themed-p">Anagram Crossword Game <br />
					<span
						onClick={() => { setShowInstructions(!showInstructions) }}
						className="show-hide">
						{showInstructions ? "Hide" : "Show"} Instructions and Description
					</span>
				</p>
				{showInstructions ? <InstructionsAndDescription /> : ""}
				<AnagramCrossword getClient={getClient} />
			</header>
		</div>
	);
}

function InstructionsAndDescription() {
	return (
		<div className="instructions-holder">
			<h3 className="instructions-h3">What is this?</h3>
			<p>This is an anagram-crossword game powered by a Rust-based crossword generation library compiled to WebAssembly for use in the browser. The entire game is offline capable once the website has loaded!
			More information about the tech running it, including the source of this website  <a className="themed-p" href="https://github.com/krhoda/wasm_crossword_generator">is found here.</a></p>
			<h3 className="instructions-h3">How do I play?</h3>
			<p>The solution to the crossword below only includes words composed of the letters presented in the bottom buttons, and includes at least one word that uses all of the letters.
				<br />
				Click the letter buttons in order to spell out a guess for the puzzle, the click the "Enter a Guess!" button when you're ready to guess.
				<br />
				A message will tell you if the guess is present in the crossword, and bad guesses are recorded below the letter buttons. The minimum guess length is 3 letters.
				<br />
				When the game is complete, you will be prompted to start a new puzzle.
				<br />
				If you get stuck you can refresh to get a new puzzle!</p>
		</div>
	)
}

function Plus() {
	return (
		<svg
			fill="#61dafb"
			version="1.1" id="Capa_1"
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 45.402 45.402"
			width="2em"
			height="2em"
		>
			<g>
				<path d="M41.267,18.557H26.832V4.134C26.832,1.851,24.99,0,22.707,0c-2.283,0-4.124,1.851-4.124,4.135v14.432H4.141
		c-2.283,0-4.139,1.851-4.138,4.135c-0.001,1.141,0.46,2.187,1.207,2.934c0.748,0.749,1.78,1.222,2.92,1.222h14.453V41.27
		c0,1.142,0.453,2.176,1.201,2.922c0.748,0.748,1.777,1.211,2.919,1.211c2.282,0,4.129-1.851,4.129-4.133V26.857h14.435
		c2.283,0,4.134-1.867,4.133-4.15C45.399,20.425,43.548,18.557,41.267,18.557z"/>
			</g>
		</svg>
	);
}

function WasmLogo() {
	return (
		<svg xmlns="http://www.w3.org/2000/svg" version="1.1"
			width="7em"
			height="7em"
			viewBox="0 0 1000 1000"
			className="wasm_svg"
		>
			<path
				d="m376 0c0 1.08 0 2.16 0 3.3 0 38.76-31.42 70.17-70.17 70.17-38.76 0-70.17-31.42-70.17-70.17l0 0c0-1.14 0-2.22 0-3.3L0 0l0 612 612 0 0-612z"
				fill="#61dafb"
			/>
			<path
				d="m142.16 329.81 40.56 0 27.69 147.47 0.5 0 33.28-147.47 37.94 0 30.06 149.28 0.59 0 31.56-149.28 39.78 0-51.69 216.69-40.25 0-29.81-147.47-0.78 0-31.91 147.47-41 0zm287.69 0 63.94 0 63.5 216.69-41.84 0-13.81-48.22-72.84 0-10.66 48.22-40.75 0zm24.34 53.41-17.69 79.5 55.06 0-20.31-79.5z"
				fill="#282c34"
			/>
		</svg>
	)
}

export default App;
