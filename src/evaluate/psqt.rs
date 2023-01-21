use crate::position::{Color, Piece, Position, Square, BOARD_SIZE};

use super::Score;

type psqt = [[i32; 8]; 8];

const MIDGAME_KNIGHT_PSQT: psqt = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

const MIDGAME_BISHOP_PSQT: psqt = [
    [-20, -10, -10, -10, -10, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 10, 5, 0, -10],
    [-10, 5, 5, 10, 10, 5, 5, -10],
    [-10, 0, 10, 10, 10, 10, 0, -10],
    [-10, 10, 10, 10, 10, 10, 10, -10],
    [-10, 5, 0, 0, 0, 0, 5, -10],
    [-20, -10, -10, -10, -10, -10, -10, -20],
];

const MIDGAME_ROOK_PSQT: psqt = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, 10, 10, 10, 10, 5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [-5, 0, 0, 0, 0, 0, 0, -5],
    [0, 0, 0, 5, 5, 0, 0, 0],
];

const MIDGAME_QUEEN_PSQT: psqt = [
    [-20, -10, -10, -5, -5, -10, -10, -20],
    [-10, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 0, -10],
    [-5, 0, 5, 5, 5, 5, 0, -5],
    [0, 0, 5, 5, 5, 5, 0, -5],
    [-10, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 0, 0, 0, 0, -10],
    [-20, -10, -10, -5, -5, -10, -10, -20],
];

const MIDGAME_KING_PSQT: psqt = [
    [20, 30, 10, 0, 0, 10, 30, 20],
    [20, 20, 0, 0, 0, 0, 20, 20],
    [-10, -20, -20, -20, -20, -20, -20, -10],
    [-20, -30, -30, -40, -40, -30, -30, -20],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
    [-30, -40, -40, -50, -50, -40, -40, -30],
];

const MIDGAME_PAWN_PSQT: psqt = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

const ENDGAME_KNIGHT_PSQT: psqt = MIDGAME_KNIGHT_PSQT;
const ENDGAME_BISHOP_PSQT: psqt = MIDGAME_BISHOP_PSQT;
const ENDGAME_ROOK_PSQT: psqt = MIDGAME_ROOK_PSQT;
const ENDGAME_QUEEN_PSQT: psqt = MIDGAME_QUEEN_PSQT;

const ENDGAME_PAWN_PSQT: psqt = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [0, 0, 0, 0, 0, 0, 0, 0],
];

const ENDGAME_KING_PSQT: psqt = [
    [-50, -30, -30, -30, -30, -30, -30, -50],
    [-30, -30, 0, 0, 0, 0, -30, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 30, 40, 40, 30, -10, -30],
    [-30, -10, 20, 30, 30, 20, -10, -30],
    [-30, -20, -10, 0, 0, -10, -20, -30],
    [-50, -40, -30, -20, -20, -30, -40, -50],
];

pub fn psqt_value(piece: &Piece<Color>, square: &Square, endgame_ratio: u8) -> Score {
    let psqt_square = match piece.color() {
        Color::Black => Square::from_row_col(7 - square.row(), square.col()),
        Color::White => *square,
    };
    let (midgame_psqt, endgame_psqt) = match piece {
        Piece::Pawn(_) => (MIDGAME_PAWN_PSQT, ENDGAME_PAWN_PSQT),
        Piece::Knight(_) => (MIDGAME_KNIGHT_PSQT, ENDGAME_KNIGHT_PSQT),
        Piece::Bishop(_) => (MIDGAME_BISHOP_PSQT, ENDGAME_BISHOP_PSQT),
        Piece::Rook(_) => (MIDGAME_ROOK_PSQT, ENDGAME_ROOK_PSQT),
        Piece::Queen(_) => (MIDGAME_QUEEN_PSQT, ENDGAME_QUEEN_PSQT),
        Piece::King(_) => (MIDGAME_KING_PSQT, ENDGAME_KING_PSQT),
    };
    let midgame_value = midgame_psqt[psqt_square.row() as usize][psqt_square.col() as usize];
    let endgame_value = endgame_psqt[psqt_square.row() as usize][psqt_square.col() as usize];
    ((midgame_value * (255 - endgame_ratio) as i32) + (endgame_value * endgame_ratio as i32)) / 255
}
