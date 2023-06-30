use crate::{
    moves::gen::piece_attacks,
    position::{
        board::{Piece, PIECES},
        CastlingRights, Color, Position,
    },
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

pub fn endgame_ratio(position: &Position) -> u8 {
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

fn evaluate_king_mobility(position: &Position, endgame_ratio: u8) -> ValueScore {
    let mut score = 0;
    let midgame_ratio = 255 - endgame_ratio as ValueScore;

    for color in &[Color::White, Color::Black] {
        let can_castle = match color {
            Color::White => {
                position.castling_rights.contains(CastlingRights::WHITE_KINGSIDE)
                    || position.castling_rights.contains(CastlingRights::WHITE_QUEENSIDE)
            }
            Color::Black => {
                position.castling_rights.contains(CastlingRights::BLACK_KINGSIDE)
                    || position.castling_rights.contains(CastlingRights::BLACK_QUEENSIDE)
            }
        };
        if can_castle {
            continue;
        }

        let king_square = (position.board.pieces_bb(Piece::King)
            & position.board.occupancy_bb(*color))
        .pop_lsb()
        .unwrap();
        let queen_attacks =
            piece_attacks(Piece::Queen, king_square, position.board.occupancy_bb_all());
        score -= std::cmp::max(10, std::cmp::min(0, queen_attacks.count_ones() as ValueScore - 3))
            * 10
            * color.sign()
            * midgame_ratio
            / 255;
    }

    score
}

pub fn evaluate_position(position: &Position) -> ValueScore {
    let mut score = 0;

    let endgame_ratio = endgame_ratio(position);

    score += evaluate_pawn_structure(position);
    score += evaluate_king_mobility(position, endgame_ratio);

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
