# Pokemon Wordle Solver

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/)

Minimize expectation.

## Demo

https://habara-k.github.io/pokemon-wordle-solver/


## Solved

| guess, answer                | optimal expectation | worst case | computation time[s] |
|------------------------------|--------------------:|-----------:|--------------------:|
| until BW(644), until BW(380) | 3.5 (=1330/380)     |          6 | 578                 |
| until XY(710), until XY(425) | 3.5576 (=1512/425)  |          6 | 2299                |
| until SM(779), until SM(474) | 3.6139 (=1713/474)  |          7 | 9474                |

excluding `ニドラン♂, ニドラン♀, ポリゴン2, ポリゴンZ`.
