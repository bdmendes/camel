use crate::position::{
    board::{Piece, PIECES, PIECES_WITH_PAWN},
    Position,
};

use super::{piece_value, psqt::psqt_value, ValueScore};

fn piece_endgame_ratio(piece: Piece) -> u8 {
    match piece {
        Piece::Pawn => 4,
        Piece::Knight => 8,
        Piece::Bishop => 8,
        Piece::Rook => 16,
        Piece::Queen => 32,
        Piece::King => 0,
    }
}

fn endgame_ratio(position: &Position) -> u8 {
    let mut endgame_ratio = u8::MAX;
    for piece in PIECES.iter() {
        let bb = position.board.pieces_bb(*piece);
        endgame_ratio =
            endgame_ratio.saturating_sub(bb.count_ones() as u8 * piece_endgame_ratio(*piece));
    }
    endgame_ratio
}

pub fn evaluate_position(position: &Position) -> ValueScore {
    let mut score = 0;

    let endgame_ratio = endgame_ratio(position);

    for piece in PIECES_WITH_PAWN.iter() {
        let mut bb = position.board.pieces_bb(*piece);
        while let Some(square) = bb.pop_lsb() {
            let color = position.board.color_at(square).unwrap();
            score += piece_value(*piece) * color.sign();
            score += psqt_value(*piece, square, color, endgame_ratio) * color.sign();
        }
    }

    score
}
