pub mod psqt;

use self::psqt::psqt_value;
use crate::position::{
    moves::{Move, MoveFlags},
    zobrist::ZobristHash,
    Color, Piece, Position, Square, BOARD_SIZE,
};

pub type Score = i32;

pub const MATE_LOWER: Score = -90000;
pub const MATE_UPPER: Score = 90000;

const CENTIPAWN_ENTROPY: Score = 10;

const fn piece_value(piece: Piece) -> Score {
    match piece {
        Piece::WP | Piece::BP => 100,
        Piece::WN | Piece::BN => 310,
        Piece::WB | Piece::BB => 320,
        Piece::WR | Piece::BR => 480,
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

pub fn evaluate_move(
    move_: &Move,
    position: &Position,
    killer_move: bool,
    hash_move: bool,
) -> Score {
    if hash_move {
        return MATE_UPPER;
    }

    if killer_move {
        return piece_value(Piece::WQ) + 1; // better than quiet moves, seemingly bad captures and promotions
    }

    let mut score: Score = 0;

    if move_.promotion.is_some() {
        score += piece_value(move_.promotion.unwrap());
    }

    let moved_piece = position.at(move_.from).unwrap();

    if move_.flags.contains(MoveFlags::CAPTURE) {
        let moved_piece_value = piece_value(moved_piece);
        let captured_piece_value = piece_value(position.at(move_.to).unwrap());
        score = 3 * captured_piece_value - moved_piece_value
            + piece_value(Piece::WQ);
    }

    let start_psqt_value = psqt_value(moved_piece, move_.from, 0);
    let end_psqt_value = psqt_value(moved_piece, move_.to, 0);
    score += end_psqt_value - start_psqt_value;

    score
}

pub fn evaluate_game_over(
    position: &Position,
    moves: &Vec<Move>,
    distance_to_root: u8,
    game_history: Option<&Vec<ZobristHash>>,
) -> Option<Score> {
    // Flag 50 move rule draws
    if position.info.half_move_number >= 100 {
        return Some(0);
    }

    // Flag 3-fold repetition draws
    if let Some(game_history) = game_history {
        let zobrist_hash = position.to_zobrist_hash();
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

pub fn evaluate_position(position: &Position, opening_entropy: bool) -> Score {
    let mut score: Score = 0;

    // Count material and midgame ratio
    let mut midgame_ratio = 0;
    for index in 0..BOARD_SIZE {
        match position.at(Square { index }) {
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
        match position.at(Square { index }) {
            None => (),
            Some(piece) => {
                let psqt_value =
                    psqt_value(piece, Square { index }, endgame_ratio);
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

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_move_heuristic_value() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        let mut moves = position.legal_moves(false);
        moves.sort_by(|a, b| {
            evaluate_move(b, &position, false, false)
                .cmp(&evaluate_move(a, &position, false, false))
        });
        assert_eq!(moves[0].to_string(), "e2a6"); // equal trade of piece
    }

    #[test]
    fn eval_checkmate() {
        let position =
            Position::from_fen("2k3R1/7R/8/8/8/4K3/8/8 b - - 0 1").unwrap();
        assert_eq!(
            evaluate_game_over(
                &position,
                &position.legal_moves(false),
                0,
                None
            )
            .unwrap(),
            MATE_LOWER
        );
    }

    #[test]
    fn eval_stalemate() {
        let position =
            Position::from_fen("8/8/8/8/8/6Q1/8/4K2k b - - 0 1").unwrap();
        assert_eq!(
            evaluate_game_over(
                &position,
                &position.legal_moves(false),
                0,
                None
            )
            .unwrap(),
            0
        );
    }

    #[test]
    fn eval_starts_zero() {
        let position = Position::new();
        assert_eq!(evaluate_position(&position, false), 0);
    }

    #[test]
    fn eval_passed_extra_pawn_midgame() {
        let position = Position::from_fen(
            "3r3k/1p1qQ1pp/p2P1n2/2p5/7B/P7/1P3PPP/4R1K1 w - - 5 26",
        )
        .unwrap();
        let evaluation = evaluate_position(&position, false);
        assert!(evaluation > 100 && evaluation < 300);
    }

    #[test]
    fn eval_forces_king_cornering() {
        let king_at_center_position =
            Position::from_fen("8/8/8/3K4/8/4q3/k7/8 b - - 6 55").unwrap();
        let king_at_corner_position =
            Position::from_fen("8/1K6/8/2q5/8/1k6/8/8 w - - 11 58").unwrap();
        let king_at_center_evaluation =
            evaluate_position(&king_at_center_position, false);
        let king_at_corner_evaluation =
            evaluate_position(&king_at_corner_position, false);
        assert!(king_at_center_evaluation > king_at_corner_evaluation);
    }
}
