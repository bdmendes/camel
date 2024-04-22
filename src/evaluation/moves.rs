use super::{psqt::psqt_value, Evaluable, ValueScore};
use crate::{moves::Move, position::Position, search::see};

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    let moving_piece = position.board.piece_at(mov.from()).unwrap();

    if mov.flag().is_capture() {
        score += see::see(mov, &position.board);
    }

    if let Some(promotion_piece) = mov.promotion_piece() {
        score += promotion_piece.value();
    }

    score += psqt_value(moving_piece, mov.to(), position.side_to_move, 0);
    score -= psqt_value(moving_piece, mov.from(), position.side_to_move, 0);

    score
}
