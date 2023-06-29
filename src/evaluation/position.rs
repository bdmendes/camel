use crate::position::{
    board::{Piece, PIECES},
    Color, Position,
};

use super::{piece_value, psqt::psqt_value, ValueScore};

fn piece_endgame_ratio(piece: Piece) -> u8 {
    match piece {
        Piece::Pawn => 4,
        Piece::Knight => 10,
        Piece::Bishop => 10,
        Piece::Rook => 16,
        Piece::Queen => 30,
        Piece::King => 0,
    }
}

fn endgame_ratio(position: &Position) -> u8 {
    let mut midgame_ratio: u8 = 0;
    for piece in PIECES.iter() {
        let bb = position.board.pieces_bb(*piece);
        midgame_ratio =
            midgame_ratio.saturating_add(bb.count_ones() as u8 * piece_endgame_ratio(*piece));
    }
    255 - midgame_ratio
}

fn evaluate_pawn_structure(position: &Position) -> ValueScore {
    let mut score = 0;
    let white_pawn_structure = position.board.pawn_structure(Color::White);
    let black_pawn_structure = position.board.pawn_structure(Color::Black);

    for file in 0..8 {
        for color in &[Color::White, Color::Black] {
            let structure =
                if *color == Color::White { &white_pawn_structure } else { &black_pawn_structure };

            let is_isolated = structure[file] > 0
                && (file == 0 || structure[file - 1] == 0)
                && (file == 7 || structure[file + 1] == 0);
            if is_isolated {
                score -= 10 * color.sign();
            }

            let doubled_penalty = match structure[file] {
                0 => 0,
                1 => 0,
                2 => 10,
                _ => 30,
            };
            score -= doubled_penalty * color.sign() * (if is_isolated { 3 } else { 1 });
        }
    }

    score
}

pub fn evaluate_position(position: &Position) -> ValueScore {
    let mut score = 0;

    let endgame_ratio = endgame_ratio(position);

    score += evaluate_pawn_structure(position);

    for piece in PIECES.iter() {
        let mut bb = position.board.pieces_bb(*piece);
        while let Some(square) = bb.pop_lsb() {
            let color = position.board.color_at(square).unwrap();
            score += piece_value(*piece) * color.sign();
            score += psqt_value(*piece, square, color, endgame_ratio) * color.sign();
        }
    }

    score
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
