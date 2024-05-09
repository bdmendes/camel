use super::{Evaluable, ValueScore};
use crate::{
    moves::Move,
    position::{board::Piece, Position},
};

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    let moving_piece = position.board.piece_at(mov.from()).unwrap();

    if mov.flag().is_capture() {
        let captured_piece = position.board.piece_at(mov.to()).unwrap_or(Piece::Pawn);
        score += captured_piece.value() - moving_piece.value() + Piece::Pawn.value();
    }

    if let Some(promotion_piece) = mov.promotion_piece() {
        score += promotion_piece.value();
    }

    score
}
