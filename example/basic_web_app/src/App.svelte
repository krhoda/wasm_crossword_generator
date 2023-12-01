<script lang="ts">
  import {CrosswordClient} from "wasm_crossword_gen";
  import type {SolutionConf, Solution} from "wasm_crossword_gen";

  import {writable} from "svelte/store";
  import type {Writable} from "svelte/store";

  let puzzle: Writable<Solution> = writable(null);
  let _puzzle: Solution = null;
  puzzle.subscribe((x) => (_puzzle = x));

  let conf: SolutionConf = {
	height: 10,
	width: 10,
	max_words: 15,
	words: [
	  {text: "finders", clue: null},
	  {text: "friends", clue: null},
	  {text: "redfins", clue: null},
	  {text: "diners", clue: null},
	  {text: "fiends", clue: null},
	  {text: "finder", clue: null},
	  {text: "friend", clue: null},
	  {text: "infers", clue: null},
	  {text: "redfin", clue: null},
	  {text: "refind", clue: null},
	  {text: "rinsed", clue: null},
	  {text: "snider", clue: null},
	  {text: "diner", clue: null},
	  {text: "dries", clue: null},
	  {text: "fends", clue: null},
	  {text: "ferns", clue: null},
	  {text: "feind", clue: null},
	  {text: "finds", clue: null},
	  {text: "fined", clue: null},
	  {text: "fired", clue: null},
	  {text: "fires", clue: null},
	  {text: "fried", clue: null},
	  {text: "fries", clue: null},
	  {text: "infer", clue: null},
	  {text: "nerds", clue: null},
	  {text: "reins", clue: null},
	  {text: "rends", clue: null},
	  {text: "resin", clue: null},
	  {text: "rides", clue: null},
	  {text: "rinse", clue: null},
	  {text: "risen", clue: null},
	  {text: "serif", clue: null},
	  {text: "sired", clue: null},
	  {text: "siren", clue: null},
	  {text: "snide", clue: null},
	  {text: "dens", clue: null},
	  {text: "dies", clue: null},
	  {text: "dine", clue: null},
	  {text: "dire", clue: null},
	  {text: "ends", clue: null},
	  {text: "feds", clue: null},
	  {text: "fend", clue: null},
	  {text: "fens", clue: null},
	  {text: "fern", clue: null},
	  {text: "find", clue: null},
	  {text: "fine", clue: null},
	  {text: "fins", clue: null},
	  {text: "fire", clue: null},
	  {text: "firs", clue: null},
	  {text: "ides", clue: null},
	  {text: "ires", clue: null},
	  {text: "nerd", clue: null},
	  {text: "refs", clue: null},
	  {text: "rein", clue: null},
	  {text: "rend", clue: null},
	  {text: "ride", clue: null},
	  {text: "rids", clue: null},
	  {text: "rife", clue: null},
	  {text: "send", clue: null},
	  {text: "side", clue: null},
	  {text: "sine", clue: null},
	  {text: "sire", clue: null},
	],
	requirements: {
	  max_retries: 100,
	  min_words: 8,
	  max_empty_columns: 1,
	  max_empty_rows: 1
	}
  };

  CrosswordClient.initialize().then((client) => {
	try {
	  let puzzle_container = client.generate_crossword_puzzle(conf, "PlacedWord");
	  console.log(puzzle_container);
	  // TODO: Update to be the full puzzle not just solution.
	  puzzle.set(puzzle_container.puzzle.solution);

	  let guess_result;
	  for (let i = 0, x = puzzle_container.puzzle.solution.words.length; i < x; i++) {
		let word = puzzle_container.puzzle.solution.words[i];
		let res = client.guess_word(puzzle_container, word);
		puzzle_container = res.puzzle_container;
		let guess_result = res.guess_result;

		if (guess_result !== "Correct" && guess_result !== "Complete") {
		  console.log("Got Bad Result:", guess_result, word)
		}
	  }

	  let res = client.is_puzzle_complete(puzzle_container);
	  puzzle_container = res.puzzle_container;
	  if (res.is_complete) {
		console.log("Success!")
		console.log(puzzle_container);
	  } else {
		console.log("Failure!")
	  }
	} catch (e) {
	  console.error(e);
	}
  });
</script>

<main>
  <p>This should be a crossword: </p>
  {#if _puzzle}
	<div class="puzzle-container">
	  {#each _puzzle.grid as row}
		<!-- <div class="puzzle-row"> -->
		{#each row.row as column}
		  <div class="puzzle-item"> {column ?? "*"} </div>
		{/each}
		<!-- </div> -->
	  {/each}
    </div>
  {/if}
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}

	.puzzle-container {
	  display: grid;
	  grid-template-columns: repeat(10, 1fr);
	}
</style>
