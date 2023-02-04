use crate::position::{moves::Move, zobrist::ZobristHash, Color, Position, BOARD_SIZE};

use super::{
    piece_midgame_ratio_gain, piece_value, psqt::psqt_value, Score, CENTIPAWN_ENTROPY, MATE_LOWER,
};

pub fn evaluate_game_over(
    position: &Position,
    moves: &Vec<Move>,
    distance_to_root: u8,
    game_history: Option<&Vec<ZobristHash>>,
) -> Option<Score> {
    // Flag 50 move rule draws
    if position.half_move_number >= 100 {
        return Some(0);
    }

    // Flag 3-fold repetition draws
    if let Some(game_history) = game_history {
        let zobrist_hash = position.zobrist_hash();
        let mut repetitions = 0;
        for hash in game_history {
            if *hash == zobrist_hash {
                repetitions += 1;
                if repetitions >= 3 {
                    return Some(0);
                }
            }
        }
    }

    // Stalemate and checkmate detection
    if moves.len() == 0 {
        let is_check = position.is_check();
        return match is_check {
            true => Some(MATE_LOWER + distance_to_root as Score),
            false => Some(0),
        };
    }

    None
}

pub fn evaluate_position(
    position: &Position,
    opening_entropy: bool,
    relative_to_current: bool,
) -> Score {
    let mut score: Score = 0;

    // Count material and midgame ratio
    let mut midgame_ratio = 0;
    for index in 0..BOARD_SIZE {
        match position.board[index] {
            None => (),
            Some(piece) => {
                let piece_value = piece_value(piece);
                score += match piece.color() {
                    Color::White => piece_value,
                    Color::Black => -piece_value,
                };
                midgame_ratio += piece_midgame_ratio_gain(piece);
            }
        }
    }
    midgame_ratio = std::cmp::min(midgame_ratio, u8::MAX as Score);
    let endgame_ratio = 255 - midgame_ratio as u8;

    // Add positional score
    for index in 0..BOARD_SIZE {
        match position.board[index] {
            None => (),
            Some(piece) => {
                let psqt_value = psqt_value(piece, index.into(), endgame_ratio);
                score += match piece.color() {
                    Color::White => psqt_value,
                    Color::Black => -psqt_value,
                };
            }
        }
    }

    // Add entropy to avoid playing the same opening moves every time
    if opening_entropy {
        score += rand::random::<Score>() % CENTIPAWN_ENTROPY;
    }

    if relative_to_current && position.to_move == Color::Black {
        -score
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_checkmate() {
        let position = Position::from_fen("2k3R1/7R/8/8/8/4K3/8/8 b - - 0 1").unwrap();
        assert_eq!(
            evaluate_game_over(&position, &position.legal_moves(false), 0, None).unwrap(),
            MATE_LOWER
        );
    }

    #[test]
    fn eval_stalemate() {
        let position = Position::from_fen("8/8/8/8/8/6Q1/8/4K2k b - - 0 1").unwrap();
        assert_eq!(
            evaluate_game_over(&position, &position.legal_moves(false), 0, None).unwrap(),
            0
        );
    }

    #[test]
    fn eval_starts_zero() {
        let position = Position::new();
        assert_eq!(evaluate_position(&position, false, false), 0);
    }

    #[test]
    fn eval_passed_extra_pawn_midgame() {
        let position =
            Position::from_fen("3r3k/1p1qQ1pp/p2P1n2/2p5/7B/P7/1P3PPP/4R1K1 w - - 5 26").unwrap();
        let evaluation = evaluate_position(&position, false, false);
        assert!(evaluation > 100 && evaluation < 300);
    }

    #[test]
    fn eval_forces_king_cornering() {
        let king_at_center_position =
            Position::from_fen("8/8/8/3K4/8/4q3/k7/8 b - - 6 55").unwrap();
        let king_at_corner_position =
            Position::from_fen("8/1K6/8/2q5/8/1k6/8/8 w - - 11 58").unwrap();
        let king_at_center_evaluation = evaluate_position(&king_at_center_position, false, false);
        let king_at_corner_evaluation = evaluate_position(&king_at_corner_position, false, false);
        assert!(king_at_center_evaluation > king_at_corner_evaluation);
    }
}
