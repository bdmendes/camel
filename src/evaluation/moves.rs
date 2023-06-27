use crate::{
    moves::Move,
    position::{board::Piece, Position},
};

use super::{piece_value, psqt::psqt_value, ValueScore};

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    if mov.flag().is_capture() {
        let captured_piece = position.board.piece_at(mov.to()).map_or_else(|| Piece::Pawn, |p| p.0);
        score += piece_value(captured_piece) + 100;
    }

    if mov.flag().is_promotion() {
        let promoted_piece = mov.promotion_piece().unwrap();
        score += match promoted_piece {
            Piece::Queen => 900,
            _ => -300,
        };
    }

    let piece = position.board.piece_at(mov.from()).unwrap().0;
    score += psqt_value(piece, mov.to(), position.side_to_move, 0);
    score -= psqt_value(piece, mov.from(), position.side_to_move, 0);

    score
}
