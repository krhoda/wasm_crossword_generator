import React from 'react';
import type { PuzzleContainer } from "wasm_crossword_generator";

export interface CrosswordProps {
	puzzleContainer?: PuzzleContainer | null
}

export default function Crossword({ puzzleContainer }: CrosswordProps) {
	if (!puzzleContainer) {
		return (<div className="puzzle_loader"><p>Loading...</p></div>);
	}
	return (
		<div className="puzzle">
			{puzzleContainer.puzzle.grid.map((r, i) => (
				<div key={i} className="puzzle_row">
					{r.row.map((s, j) => (
						<div key={j} className={`puzzle_space ${s.has_char_slot ? "puzzle_space_char" : "puzzle_space_blank"}`}>{s.has_char_slot ? (s.char_slot ? s.char_slot.toUpperCase() : " ") : ""}</div>
					))}
				</div>
			))
			}
		</div>
	)
}
