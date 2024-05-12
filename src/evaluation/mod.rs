use crate::position::{board::Piece, Color};

pub mod moves;
pub mod position;
pub mod psqt;

pub type ValueScore = i16;

// Mate values are in the range ]MIN+200, MIN+400] and ]MAX-400, MAX-200].
// 200 is an arbitrary value that is large enough to not interfere
// with regular scores or alpha and beta bounds and fit very long lines.
const MATE_SCORE_THRESHOLD: ValueScore = 200;
pub const MATE_SCORE: ValueScore = ValueScore::MIN + 200;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Score {
    Mate(Color, u8),
    Value(ValueScore),
}

impl Score {
    pub fn is_mate(score: ValueScore) -> bool {
        !((MATE_SCORE + MATE_SCORE_THRESHOLD)..=(MATE_SCORE.abs() - MATE_SCORE_THRESHOLD))
            .contains(&score)
    }
}

pub trait Evaluable {
    fn value(&self) -> ValueScore;
}

impl Evaluable for Piece {
    fn value(&self) -> ValueScore {
        match self {
            Piece::Pawn => 100,
            Piece::Knight => 310,
            Piece::Bishop => 330,
            Piece::Rook => 480,
            Piece::Queen => 900,
            Piece::King => 6000,
        }
    }
}
