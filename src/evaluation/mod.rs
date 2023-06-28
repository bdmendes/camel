use crate::position::{board::Piece, Color};

pub mod moves;
pub mod position;
mod psqt;

pub type ValueScore = i16;

pub enum Score {
    Mate(Color, u8),
    Value(ValueScore),
}

pub fn piece_value(piece: Piece) -> ValueScore {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}
