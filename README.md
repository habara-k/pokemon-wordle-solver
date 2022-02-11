# Pokemon Wordle Solver

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](http://creativecommons.org/publicdomain/zero/1.0/)

Minimize expectation.

## Demo

https://habara-k.github.io/pokemon-wordle-solver/


## Solved

| mode (n_ans)   | optimal expectation | worst case | computation time[s] |
|----------------|--------------------:|-----------:|--------------------:|
| until DP(282)  | 3.3404 (= 942/282)  | 6          | 95                  |
| until BW(380)  | 3.4947 (= 1328/380) | 6          | 1051                |
| until XY(425)  | 3.5576 (= 1512/425) | 6          | 3497                |
| until SM(474)  | 3.6139 (= 1713/474) | 7          | 11488               |
| until SWSH(511)| 3.6379 (= 1859/511) | 6          | 24891               |
