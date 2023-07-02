<div align="center">
<img src="readme_assets/camel.svg" width="250">
<br>
<br>

[![](https://img.shields.io/github/actions/workflow/status/bdmendes/camel/rust.yml?style=for-the-badge)](https://github.com/bdmendes/camel/actions)
[![](https://img.shields.io/github/v/release/bdmendes/camel?style=for-the-badge)](https://github.com/bdmendes/camel/releases)

</div>

## Overview

Camel is a chess engine written from scratch in Rust. It aims to achieve a high level of play (>2000 [Elo](https://en.wikipedia.org/wiki/Elo_rating_system)), while also being easy to understand and modify.

Camel is on [lichess](https://lichess.org/@/camel_bot), through the [lichess-bot bridge](https://github.com/lichess-bot-devs/lichess-bot).

## Testing

You can probe the integrity of the engine by running the test suite, which includes [perft](https://www.chessprogramming.org/Perft_Results) and other unit tests:

<pre>
    cargo test
</pre>

Upon developing, to be able to claim a statistically significant improvement over the last version, it is recommended to setup a [tournament](https://www.chessprogramming.org/Chess_Tournaments) between the two versions, using an utility such as [fast-chess](https://github.com/Disservin/fast-chess). It is also possible and fun to deploy the engine to [lichess](https://lichess.org/), through the [lichess-bot bridge](https://github.com/lichess-bot-devs/lichess-bot), although the [Elo](https://en.wikipedia.org/wiki/Elo_rating_system) might not represent the engine's true strength, since it will mostly be based on matchmaking against other engines.

## How to use it

Camel is an [UCI-compatible](https://backscattering.de/chess/uci/) chess engine, which means it can be used with any chess GUI that supports the UCI protocol, such as [Scid](https://flathub.org/apps/details/io.github.benini.scid). Alternatively, you can explore it through the interactive CLI, which builds on top of the UCI protocol, allowing you to visualize the board, make a move of your own, query legal moves and ask the engine to move.

<pre>
$ <b>cargo run --release</b>
$ <b>position fen r1q1k1r1/pp1np1b1/5npp/1Q1NN1p1/3P4/4B2P/PPP2PP1/4RRK1 w q - 3 16</b>
$ <b>display</b>
♜ - ♛ - ♚ - ♜ - 
♟ ♟ - ♞ ♟ - ♝ - 
- - - - - ♞ ♟ ♟ 
- ♕ - ♘ ♘ - ♟ - 
- - - ♙ - - - - 
- - - - ♗ - - ♙ 
♙ ♙ ♙ - - ♙ ♙ - 
- - - - ♖ ♖ ♔ - 
$ <b>list</b>
e1e2 e1d1 e1c1 e1b1 e1a1 g1h1 g1h2 a2a3 a2a4 b2b3 b2b4 c2c3 c2c4 f2f3 f2f4 g2g3 g2g4 e3f4 e3g5 e3d2 e3c1 h3h4 b5b6 b5b7 b5b4 b5b3 b5a5 b5c5 b5a6 b5c6 b5d7 b5a4 b5c4 b5d3 b5e2 d5c7 d5e7 d5c3 d5b6 d5b4 d5f6 d5f4 e5d7 e5f7 e5d3 e5f3 e5c6 e5c4 e5g6 e5g4
$ <b>go depth 6</b>
info depth 1 score cp 324 time 4 nodes 598 nps 137134 pv e5g6
info depth 2 score cp 324 time 13 nodes 598 nps 42912 pv e5g6 e7e6
info depth 3 score cp 309 time 57 nodes 1715 nps 29579 pv e5g6 e7e6 g6e7
info depth 4 score cp 309 time 226 nodes 4812 nps 21237 pv e5g6 e7e6 g6e7 a7a6
info depth 5 score cp 307 time 1166 nodes 19830 nps 16992 pv e5g6 e7e6 g6e7 a7a6 b5d3
info depth 6 score cp 312 time 3716 nodes 112194 nps 30189 pv e3g5 h6g5 e5d7 f6d7 d5e7 g7d4
bestmove e3g5
$ <b>move e3g5</b>
♜ - ♛ - ♚ - ♜ - 
♟ ♟ - ♞ ♟ - ♝ - 
- - - - - ♞ ♟ ♟ 
- ♕ - ♘ ♘ - ♗ - 
- - - ♙ - - - - 
- - - - - - - ♙ 
♙ ♙ ♙ - - ♙ ♙ - 
- - - - ♖ ♖ ♔ - 
</pre>

Type `help` to see the available commands.

## How it works

Camel is a classical engine, such as [Stockfish](https://stockfishchess.org/). It searches the game tree using [principal variation search](https://www.chessprogramming.org/Principal_Variation_Search), an enhancement to the regular [minimax algorithm](https://en.wikipedia.org/wiki/Minimax) with [alpha-beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning), made possible by the move ordering the [iterative deepening](https://www.chessprogramming.org/Iterative_Deepening) framework provides. At the end of the regular search, the engine performs a [quiescence search](https://www.chessprogramming.org/Quiescence_Search) to diminish the [horizon effect](https://www.chessprogramming.org/Horizon_Effect).

For efficiency reasons, candidate moves are [ordered](https://www.chessprogramming.org/Move_Ordering) carefully, and the same positions are not searched again, thanks to [transposition tables](https://www.chessprogramming.org/Transposition_Table). Basic positional knowledge is provided by the use of [piece-square tables](https://www.chessprogramming.org/Piece-Square_Tables), which are used in a [tapered](https://www.chessprogramming.org/Tapered_Eval) way to account for endgames positions.

Chess rules are implemented carefully and extensively, including details such as draw by threefold repetition and 50 quiet moves. The move generation module is tested with [perft](https://www.chessprogramming.org/Perft).

[Chess programming](https://www.chessprogramming.org/Main_Page) is a vast topic, and Camel is far from being a complete engine. It has great room for improvement, both in terms of search algorithms and move generation performance. If you are interested in chess programming, I encourage you to read the links above and contribute to this project. Feel free to fork the repository and open a pull request.

## What can I do with it?

Camel is licensed under the [GNU General Public License v3.0](./LICENSE.md). You can use it for any purpose, including commercial use, provided you always include the license and the source code.

## Credits

- [@biromiro](https://github.com/biromiro): for designing the cute camel logo.
- [Chess Programming Wiki](https://www.chessprogramming.org/Main_Page): a great resource for chess programming.

## Why Camel?

Camel likes to play active chess. It does grunt sometimes when it loses, though.