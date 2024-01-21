import React, { useEffect, useState } from 'react';
import logo from './logo.svg';
import './App.css';
import Crossword from "./components/Crossword";
import { CrosswordClient, PuzzleContainer, SolutionConf } from "wasm_crossword_gen";
import { hardcoded_conf } from "./util/crossword_conf";

const initPuzzleContainer: PuzzleContainer | null = null;

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

async function newPuzzle(conf: SolutionConf): Promise<PuzzleContainer> {
	let client = await getClient();
	return client.generate_crossword_puzzle(
		conf,
		"PlacedWord",
	);
}

function App() {
	let [puzzleContainer, setPuzzleContainer] = useState(initPuzzleContainer);
	useEffect(() => {
		newPuzzle(hardcoded_conf).then((pc) => {
			setPuzzleContainer(pc);
			console.log(pc);
		}).catch((e) => {
			// TODO: Make Err Container
			console.error(e);
		})

	}, [])


	return (
		<div className="App">
			<header className="App-header">
				<img src={logo} className="App-logo" alt="logo" />
				<Crossword puzzleContainer={puzzleContainer} />
			</header>
		</div>
	);
}

export default App;
