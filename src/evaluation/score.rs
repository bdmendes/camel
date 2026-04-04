use std::fmt::Display;

pub type ValueScore = i16;

pub const MATE_SCORE: ValueScore = ValueScore::MIN + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(MATE_SCORE, Score::Mate(0))]
    #[case(MATE_SCORE + 1, Score::Mate(1))]
    #[case(MATE_SCORE + 2, Score::Mate(1))]
    #[case(MATE_SCORE + 3, Score::Mate(2))]
    #[case(-MATE_SCORE, Score::Mate(0))]
    #[case(-MATE_SCORE - 1, Score::Mate(1))]
    #[case(-MATE_SCORE - 2, Score::Mate(1))]
    #[case(-MATE_SCORE - 3, Score::Mate(2))]
    #[case(0, Score::Value(0))]
    #[case(1200, Score::Value(1200))]
    #[case(-1200, Score::Value(-1200))]
    fn parse(#[case] value: ValueScore, #[case] expected: Score) {
        let score = Score::from(value);
        assert_eq!(score, expected);
        assert_eq!(Score::is_mate(value), matches!(score, Score::Mate(_)))
    }
}
