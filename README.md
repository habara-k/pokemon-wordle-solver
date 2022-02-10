# Pokemon Wordle Solver

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/)

Minimize expectation.

## Demo

https://habara-k.github.io/pokemon-wordle-solver/


## Solved

| mode (pokemons with <=5, 5 words) | optimal expectation | worst case | computation time[s] |
|-----------------------------------|---------------------|------------|---------------------|
| until BW (644, 380)               | 3.5 (=1330/380)     | 6          | 578                 |
| until XY (710, 425)               | 3.5576 (=1512/425)  | 6          | 2299                |

excluding `ニドラン♂, ニドラン♀, ポリゴン2, ポリゴンZ`.
