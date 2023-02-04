use crate::position::{
    moves::{pseudo_legal_moves_from_square, Move},
    zobrist::ZobristHash,
    CastlingRights, Color, Piece, Position, Square, BOARD_SIZE,
};

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

fn evaluate_pawn_structure(position: &Position) -> Score {
    let mut white_pawns: [u8; 8] = [0; 8];
    let mut black_pawns: [u8; 8] = [0; 8];

    let mut score = 0;

    for index in 0..BOARD_SIZE {
        match position.board[index] {
            Some(Piece::WP) => {
                let col = index % 8;
                white_pawns[col] += 1;
            }
            Some(Piece::BP) => {
                let col = index % 8;
                black_pawns[col] += 1;
            }
            _ => (),
        }
    }

    for col in 0..8 {
        // Penalty for doubled pawns
        score -= match white_pawns[col] {
            0 => 0,
            1 => 0,
            2 => 20,
            _ => 40,
        };
        score += match black_pawns[col] {
            0 => 0,
            1 => 0,
            2 => 20,
            _ => 40,
        };

        // Penalty for isolated pawns
        if white_pawns[col] > 0 {
            if col == 0 {
                if white_pawns[col + 1] == 0 {
                    score -= 10;
                }
            } else if col == 7 {
                if white_pawns[col - 1] == 0 {
                    score -= 10;
                }
            } else {
                if white_pawns[col - 1] == 0 && white_pawns[col + 1] == 0 {
                    score -= 10;
                }
            }
        }
        if black_pawns[col] > 0 {
            if col == 0 {
                if black_pawns[col + 1] == 0 {
                    score += 10;
                }
            } else if col == 7 {
                if black_pawns[col - 1] == 0 {
                    score += 10;
                }
            } else {
                if black_pawns[col - 1] == 0 && black_pawns[col + 1] == 0 {
                    score += 10;
                }
            }
        }
    }

    score
}

fn king_mobility(position: &Position, king_square: Square, color: Color) -> Score {
    let faker_queen = match color {
        Color::White => Piece::WQ,
        Color::Black => Piece::BQ,
    };
    let faker_moves =
        pseudo_legal_moves_from_square(position, king_square, false, Some(faker_queen));
    faker_moves.len() as Score
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
    let midgame_ratio = midgame_ratio as u8;
    let endgame_ratio = 255 - midgame_ratio;

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

    // Add king safety
    for index in 0..BOARD_SIZE {
        match position.board[index] {
            Some(Piece::WK)
                if !position.castling_rights.intersects(
                    CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE,
                ) =>
            {
                let king_mobility = king_mobility(position, index.into(), Color::White);
                score -= king_mobility * 20 * (midgame_ratio as Score) / 255;
            }
            Some(Piece::BK)
                if !position.castling_rights.intersects(
                    CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE,
                ) =>
            {
                let king_mobility = king_mobility(position, index.into(), Color::Black);
                score += king_mobility * 20 * (midgame_ratio as Score) / 255;
            }
            _ => (),
        }
    }

    // Add pawn structure considerations
    score += evaluate_pawn_structure(position);

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
    fn eval_king_mobility() {
        let position = Position::from_fen("1k6/p1p2p1p/P7/2P5/2KBrnP1/7P/8/7R w - - 1 36").unwrap();
        assert_eq!(king_mobility(&position, Square::from_algebraic("c4"), Color::White), 14);
    }

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
