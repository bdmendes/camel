use crate::position::{board::Piece, square::Square, Color};

use super::ValueScore;

type PieceSquareTable = [ValueScore; 64];

// Values adapted from https://www.chessprogramming.org/Simplified_Evaluation_Function
// The board is reversed (white is at the bottom) to allow for easier tuning.
// Values range from -50 to 50, meaning that a good piece placement is worth at most 50 centipawns.

#[rustfmt::skip]
const MIDGAME_KNIGHT_PSQT: PieceSquareTable = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -30, -5, 15, 20, 20, 15, -5,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-35,-30,-30,-30,-30,-35,-50,
];

#[rustfmt::skip]
const MIDGAME_BISHOP_PSQT: PieceSquareTable = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10, 0, -10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const MIDGAME_ROOK_PSQT: PieceSquareTable = [
     0, 0, 0, 0, 0, 0, 0, 0,
     5,10,10,20,20,10,10, 5,
    -5, 0, 0, 0, 0, 0, 0,-5,
    -5, 0, 0, 0, 0, 0, 0,-5,
    -5, 0, 0, 0, 0, 0, 0,-5,
    -5, 0, 0, 0, 0, 0, 0,-5,
    -5, 0, 0, 5, 5, 0, 0,-5,
     0, 0, 0,10,10, 0, 0, 0,
];

#[rustfmt::skip]
const MIDGAME_QUEEN_PSQT: PieceSquareTable = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const MIDGAME_KING_PSQT: PieceSquareTable = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
     20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
      5,  5,-10,-10,-10,-10,  5, 10,
     10, 20, 15,-10,-10, 10, 25, 10,
];

#[rustfmt::skip]
const MIDGAME_PAWN_PSQT: PieceSquareTable = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 15, 15, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10, 15, 15,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

const ENDGAME_KNIGHT_PSQT: PieceSquareTable = MIDGAME_KNIGHT_PSQT;
const ENDGAME_BISHOP_PSQT: PieceSquareTable = MIDGAME_BISHOP_PSQT;
const ENDGAME_ROOK_PSQT: PieceSquareTable = MIDGAME_ROOK_PSQT;
const ENDGAME_QUEEN_PSQT: PieceSquareTable = MIDGAME_QUEEN_PSQT;

#[rustfmt::skip]
const ENDGAME_KING_PSQT: PieceSquareTable = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
];

#[rustfmt::skip]
const ENDGAME_PAWN_PSQT: PieceSquareTable = [
      0,  0,  0,  0,  0,  0,  0,  0,
    110,120,120,120,120,120,120,110,
     90,100,100,100,100,100,100, 90,
     70, 80, 80, 80, 80, 80, 80, 70,
     60, 60, 60, 60, 60, 60, 60, 60,
     40, 40, 40, 40, 40, 40, 40, 40,
     20, 20, 20, 20, 20, 20, 20, 20,
      0,  0,  0,  0,  0,  0,  0,  0,
];

pub fn psqt_value(piece: Piece, square: Square, color: Color, endgame_ratio: u8) -> ValueScore {
    let midgame_psqt = match piece {
        Piece::Pawn => &MIDGAME_PAWN_PSQT,
        Piece::Knight => &MIDGAME_KNIGHT_PSQT,
        Piece::Bishop => &MIDGAME_BISHOP_PSQT,
        Piece::Rook => &MIDGAME_ROOK_PSQT,
        Piece::Queen => &MIDGAME_QUEEN_PSQT,
        Piece::King => &MIDGAME_KING_PSQT,
    };

    let endgame_psqt = match piece {
        Piece::Pawn => &ENDGAME_PAWN_PSQT,
        Piece::Knight => &ENDGAME_KNIGHT_PSQT,
        Piece::Bishop => &ENDGAME_BISHOP_PSQT,
        Piece::Rook => &ENDGAME_ROOK_PSQT,
        Piece::Queen => &ENDGAME_QUEEN_PSQT,
        Piece::King => &ENDGAME_KING_PSQT,
    };

    let square = match color {
        Color::White => square.flip() as usize,
        Color::Black => square as usize,
    };
    let midgame_value = midgame_psqt[square];
    let endgame_value = endgame_psqt[square];

    let endgame_ratio = endgame_ratio as ValueScore;
    (midgame_value * (255 - endgame_ratio) + endgame_value * endgame_ratio) / 255
}
