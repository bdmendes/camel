# Camel Chess Engine

![Camel Chess Engine](./readme_assets/camel.png)

Camel is a chess engine written from scratch in Rust. It aims to achieve a high level of play (>2000 [Elo](https://en.wikipedia.org/wiki/Elo_rating_system), although not yet tested extensively), while also being easy to understand and modify.

## How to use it

Camel is an [UCI-compatible](https://backscattering.de/chess/uci/) chess engine, which means it can be used with any chess GUI that supports the UCI protocol, such as [Scid](https://flathub.org/apps/details/io.github.benini.scid). Alternatively, you can explore it through the interactive CLI, which builds on top of the UCI protocol, allowing you to visualize the board, make a move of your own, query legal moves and ask the engine to move. Type `help` to see the available commands.

<pre>
$ <b>cargo run</b>
display
♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 
♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 
- - - - - - - - 
- - - - - - - - 
- - - - - - - - 
- - - - - - - - 
♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 
♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖ 

list
b1a3 b1c3 g1f3 g1h3 a2a3 a2a4 b2b3 b2b4 c2c3 c2c4 d2d3 d2d4 e2e3 e2e4 f2f3 f2f4 g2g3 g2g4 h2h3 h2h4 

automove
info depth 1 score cp 50 time 4 nodes 20 nps 3841 pv b1c3
info depth 2 score cp 0 time 23 nodes 39 nps 1599 pv b1c3 b8c6
info depth 3 score cp 50 time 56 nodes 464 nps 8084 pv b1c3 b8c6 g1f3
info depth 4 score cp 0 time 266 nodes 941 nps 3523 pv b1c3 b8c6 g1f3 g8f6
bestmove b1c3
♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜ 
♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟ 
- - - - - - - - 
- - - - - - - - 
- - - - - - - - 
- - ♘ - - - - - 
♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙ 
♖ - ♗ ♕ ♔ ♗ ♘ ♖ 
</pre>

## How it works

Camel is a classical engine, such as [Stockfish](https://stockfishchess.org/). It searches the game tree using the [minimax algorithm](https://en.wikipedia.org/wiki/Minimax), with [alpha-beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning), evaluating the position statically at a leaf node. The engine uses [quiescence search](https://www.chessprogramming.org/Quiescence_Search) to diminish the [horizon effect](https://www.chessprogramming.org/Horizon_Effect).

For efficiency reasons, candidate moves are [ordered](https://www.chessprogramming.org/Move_Ordering) carefully, and the same positions are not searched again, thanks to [transposition tables](https://www.chessprogramming.org/Transposition_Table). Basic positional knowledge is provided by the use of [piece-square tables](https://www.chessprogramming.org/Piece-Square_Tables), which are used in a [tapered](https://www.chessprogramming.org/Tapered_Eval) way to account for endgames positions.

Chess rules are implemented carefully and extensively, including details such as draw by threefold repetition and 50 quiet moves. The move generation module is tested with [perft](https://www.chessprogramming.org/Perft).

[Chess programming](https://www.chessprogramming.org/Main_Page) is a vast topic, and Camel is far from being a complete engine. It has great room for improvement, both in terms of search algorithms and move generation performance. If you are interested in chess programming, I encourage you to read the links above, and contribute to this project.

## Why Camel?

Camel likes to play active chess. It does grunt sometimes when it loses, though.