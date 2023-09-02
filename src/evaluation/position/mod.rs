use self::pawns::evaluate_pawn_structure;

use super::{piece_value, psqt::psqt_value, ValueScore};
use crate::position::{board::Piece, Position};

mod pawns;

pub const MAX_POSITIONAL_GAIN: ValueScore = 200;

pub fn piece_midgame_ratio(piece: Piece) -> u8 {
    match piece {
        Piece::Pawn => 0,
        Piece::Knight => 10,
        Piece::Bishop => 10,
        Piece::Rook => 16,
        Piece::Queen => 32,
        Piece::King => 0,
    }
}

pub fn midgame_ratio(position: &Position) -> u8 {
    let mut ratio: u8 = 0;
    for piece in Piece::list() {
        ratio = ratio.saturating_add(
            position.board.pieces_bb(*piece).count_ones() as u8 * piece_midgame_ratio(*piece),
        );
    }
    ratio
}

pub fn evaluate_position(position: &Position) -> ValueScore {
    let endgame_ratio = 255 - midgame_ratio(position);
    let mut score = 0;

    for piece in Piece::list() {
        let piece_value = piece_value(*piece);

        for square in position.board.pieces_bb(*piece) {
            let color = position.board.color_at(square).unwrap();
            let color_sign = color.sign();

            // Material score
            score += piece_value * color_sign;

            // Positional score
            let psqt_score = psqt_value(*piece, square, color, endgame_ratio);
            score += psqt_score * color_sign;
        }
    }

    score + evaluate_pawn_structure(position)
}

#[cfg(test)]
mod tests {
    use crate::position::{fen::START_FEN, Position};

    #[test]
    fn eval_starts_zero() {
        let position = Position::from_fen(START_FEN).unwrap();
        assert_eq!(super::evaluate_position(&position), 0);
    }

    #[test]
    fn eval_passed_extra_pawn_midgame() {
        let position =
            Position::from_fen("3r3k/1p1qQ1pp/p2P1n2/2p5/7B/P7/1P3PPP/4R1K1 w - - 5 26").unwrap();
        let evaluation = super::evaluate_position(&position);
        assert!(evaluation > 100 && evaluation < 300);
    }

    #[test]
    fn eval_forces_king_cornering() {
        let king_at_center_position =
            Position::from_fen("8/8/8/3K4/8/4q3/k7/8 b - - 6 55").unwrap();
        let king_at_corner_position =
            Position::from_fen("8/1K6/8/2q5/8/1k6/8/8 w - - 11 58").unwrap();
        let king_at_center_evaluation = super::evaluate_position(&king_at_center_position);
        let king_at_corner_evaluation = super::evaluate_position(&king_at_corner_position);
        assert!(king_at_center_evaluation > king_at_corner_evaluation);
    }
}
