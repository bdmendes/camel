use std::fmt::Display;

pub type ValueScore = i16;

pub const MATE_SCORE: ValueScore = ValueScore::MIN + 2;

pub enum Score {
    Value(ValueScore),
    Mate(u8),
}

impl From<ValueScore> for Score {
    fn from(value: ValueScore) -> Self {
        if Score::is_mate(value) {
            let mate_in = (MATE_SCORE.abs() - value.abs() + 1) / 2;
            Score::Mate((mate_in & 0xFF) as u8)
        } else {
            Score::Value(value)
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Score::Value(value) => write!(f, "cp {}", value),
            Score::Mate(mate_in) => write!(f, "mate {}", mate_in),
        }
    }
}

impl Score {
    pub fn is_mate(value: ValueScore) -> bool {
        value.abs() >= (MATE_SCORE + u8::MAX as i16).abs()
    }
}
