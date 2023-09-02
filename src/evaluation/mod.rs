use crate::position::{board::Piece, Color};

pub mod moves;
pub mod position;

pub type ValueScore = i16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Score {
    Mate(Color, u8),
    Value(ValueScore),
}

pub fn piece_value(piece: Piece) -> ValueScore {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 310,
        Piece::Bishop => 330,
        Piece::Rook => 480,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}
