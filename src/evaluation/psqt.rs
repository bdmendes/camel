use super::ValueScore;
use crate::position::{board::Piece, square::Square, Color};

type PieceSquareTable = [ValueScore; 64];

// Values adapted from https://www.chessprogramming.org/Simplified_Evaluation_Function
// The board is reversed (white is at the bottom) to allow for easier tuning.

#[rustfmt::skip]
const MIDGAME_KNIGHT_PSQT: PieceSquareTable = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
const MIDGAME_BISHOP_PSQT: PieceSquareTable = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
const MIDGAME_ROOK_PSQT: PieceSquareTable = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
const MIDGAME_QUEEN_PSQT: PieceSquareTable = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
const MIDGAME_KING_PSQT: PieceSquareTable = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
const MIDGAME_PAWN_PSQT: PieceSquareTable = [
    0,   0,   0,   0,   0,   0,  0,   0,
    98, 134,  61,  95,  68, 126, 34, -11,
    -6,   7,  26,  31,  65,  56, 25, -20,
   -14,  13,   6,  21,  23,  12, 17, -23,
   -27,  -2,  -5,  12,  17,   6, 10, -25,
   -26,  -4,  -4, -10,   3,   3, 33, -12,
   -35,  -1, -20, -23, -15,  24, 38, -22,
     0,   0,   0,   0,   0,   0,  0,   0,
];

const ENDGAME_KNIGHT_PSQT: PieceSquareTable = MIDGAME_KNIGHT_PSQT;
const ENDGAME_BISHOP_PSQT: PieceSquareTable = MIDGAME_BISHOP_PSQT;
const ENDGAME_ROOK_PSQT: PieceSquareTable = MIDGAME_ROOK_PSQT;
const ENDGAME_QUEEN_PSQT: PieceSquareTable = MIDGAME_QUEEN_PSQT;

#[rustfmt::skip]
const ENDGAME_KING_PSQT: PieceSquareTable = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

#[rustfmt::skip]
const ENDGAME_PAWN_PSQT: PieceSquareTable = [
    0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
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

    let endgame_ratio = endgame_ratio as i32;
    ((midgame_value as i32 * (255 - endgame_ratio) + endgame_value as i32 * endgame_ratio) / 255)
        as ValueScore
}
