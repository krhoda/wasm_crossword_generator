import React from 'react';
import type { PuzzleContainer } from "wasm_crossword_gen";

export interface CrosswordProps {
	puzzleContainer?: PuzzleContainer | null
}

export default function Crossword({ puzzleContainer }: CrosswordProps) {
	if (!puzzleContainer) {
		return (<p>No puzzle provided, puzzle may be loading...</p>);
	}
	return (
		<div className="puzzle">
			{puzzleContainer.puzzle.grid.map((r, i) => (
				<div key={i} className="puzzle_row">
					{r.row.map((s, j) => (
						<div key={j} className={`puzzle_space ${s.has_char_slot ? "puzzle_space_char" : "puzzle_space_blank"}`}>{s.has_char_slot ? s.char_slot ?? "a" : ""}</div>
					))}
				</div>
			))
			}
		</div>
	)
}
