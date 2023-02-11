pub mod moves;
pub mod position;
pub mod psqt;

use crate::position::Piece;

pub type Score = i32;

pub const MATE_LOWER: Score = -90000;
pub const MATE_UPPER: Score = 90000;

const CENTIPAWN_ENTROPY: Score = 5;

pub fn piece_value(piece: Piece) -> Score {
    // Values adapted from https://www.chessprogramming.org/Simplified_Evaluation_Function
    match piece {
        Piece::WP | Piece::BP => 100,
        Piece::WN | Piece::BN => 310,
        Piece::WB | Piece::BB => 320,
        Piece::WR | Piece::BR => 500,
        Piece::WQ | Piece::BQ => 900,
        _ => 0,
    }
}

fn piece_midgame_ratio_gain(piece: Piece) -> Score {
    // Values engineered so that they add up to 255, the ratio to interpolate
    // between the midgame and endgame PSQT tables
    // (2×8 + 10×2 + 10×2 + 16×2 + 39)×2 = 254
    match piece {
        Piece::WP | Piece::BP => 2,
        Piece::WN | Piece::BN => 10,
        Piece::WB | Piece::BB => 10,
        Piece::WR | Piece::BR => 16,
        Piece::WQ | Piece::BQ => 39,
        _ => 0,
    }
}
