<script lang="ts">
  import {CrosswordClient} from "wasm_crossword_gen";
  import type {CrosswordConf} from "wasm_crossword_gen";

  let conf: CrosswordConf = {
	height: 10,
	width: 10,
	max_words: 2,
	words: ["canine", "cat"]
  };

  CrosswordClient.initialize().then((client) => {
	let puzzle = client.generate_crossword_puzzle(conf);
	console.log(puzzle);
  })
</script>

<main>
  <p>Check the console log for the puzzle that will be rendered one day!</p>
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
