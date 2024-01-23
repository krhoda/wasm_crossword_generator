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
	return (
		<div className="App">
			<header className="App-header">
				<img src={logo} className="App-logo" alt="logo" />
				<p>React + WebAssembly Crossword Games</p>
				<AnagramCrossword getClient={getClient} />
			</header>
		</div>
	);
}

// TODO: Add Header:
/*
<div className="button_container">
	<button className="nav_button">Anagram-Based<br />Simple Crossword</button>
	<button className="nav_button">Classic<br />Crossword</button>
</div>
*/

export default App;
