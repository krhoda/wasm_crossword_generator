# Create-React-App + WASM Anagram Crossword Game

This directory contains a working demo using the `wasm_crossword_generator` npm package. If you have npm installed, you can simply clone this repository, open a terminal in this directory and run:
```
$ npm i
$ npm start
```
Then visit `localhost:3000` in the browser. There you will see a basic crossword game (a "PerWord" playmode). It uses anagrams to create the crossword, and thus doesn't have clues, just a set of possible letters the answers are made out of. More detailed instructions are available in the app.

This also functions as a demo for other applications that might want to use `wasm_crossword_generator`. Both [App.tsx](src/App.tsx) and [AnagramCrossword](src/components/AnagramCrossword.tsx) contain useful example, the former showing how to deal with the asynchronous initialization required by the `CrosswordClient` and the latter showing puzzle generation and guess handling. Also, this demonstrates how easy it is to use the WASM library even in a circumstance where a end-user doesn't control the build system, such as the default Create-React-App.
