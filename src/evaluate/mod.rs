mod psqt;

use crate::position::{Color, Piece, Position, Square, BOARD_SIZE};

use self::psqt::psqt_value;

pub type Score = i32;
pub type Evaluation = f32;

pub fn piece_value(piece: Piece) -> Score {
    // Values from https://github.com/official-stockfish/Stockfish/blob/master/src/types.h
    match piece {
        Piece::Pawn(_) => 100,
        Piece::Knight(_) => 310,
        Piece::Bishop(_) => 320,
        Piece::Rook(_) => 480,
        Piece::Queen(_) => 900,
        _ => 0,
    }
}

fn piece_midgame_ratio_gain(piece: Piece) -> Score {
    // Values engineered so that they add up to 255, the ratio to interpolate
    // between the midgame and endgame PSQT tables
    // (2×8 + 10×2 + 10×2 + 16×2 + 39)×2 = 254
    match piece {
        Piece::Pawn(_) => 2,
        Piece::Knight(_) => 10,
        Piece::Bishop(_) => 10,
        Piece::Rook(_) => 16,
        Piece::Queen(_) => 39,
        _ => 0,
    }
}

pub fn evaluate_position(position: &Position) -> Evaluation {
    let mut score: Score = 0;

    // Count material and midgame ratio
    let mut midgame_ratio = 0;
    for index in 0..BOARD_SIZE {
        match position.at(&Square { index }) {
            None => (),
            Some(piece) => {
                let piece_value = piece_value(piece);
                score += match piece.color() {
                    Color::White => piece_value,
                    Color::Black => -piece_value,
                };
                midgame_ratio += piece_midgame_ratio_gain(piece) as u8;
            }
        }
    }

    // Add positional score
    for index in 0..BOARD_SIZE {
        match position.at(&Square { index }) {
            None => (),
            Some(piece) => {
                let psqt_value = psqt_value(piece, Square { index }, 255 - midgame_ratio);
                score += match piece.color() {
                    Color::White => psqt_value,
                    Color::Black => -psqt_value,
                };
            }
        }
    }

    score as f32 / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_starts_zero() {
        let position = Position::new();
        assert_eq!(evaluate_position(&position), 0.0);
    }

    #[test]
    fn eval_passed_extra_pawn_midgame() {
        let position =
            Position::from_fen("3r3k/1p1qQ1pp/p2P1n2/2p5/7B/P7/1P3PPP/4R1K1 w - - 5 26").unwrap();
        let evaluation = evaluate_position(&position);
        assert!(evaluation > 1.0 && evaluation < 3.0);
    }

    #[test]
    fn eval_forces_king_cornering() {
        let king_at_center_position =
            Position::from_fen("8/8/8/3K4/8/4q3/k7/8 b - - 6 55").unwrap();
        let king_at_corner_position =
            Position::from_fen("8/1K6/8/2q5/8/1k6/8/8 w - - 11 58").unwrap();
        let king_at_center_evaluation = evaluate_position(&king_at_center_position);
        let king_at_corner_evaluation = evaluate_position(&king_at_corner_position);
        assert!(king_at_center_evaluation < 8.0);
        assert!(king_at_corner_evaluation < 8.0);
        assert!(king_at_center_evaluation > king_at_corner_evaluation);
    }
}
