use super::{psqt::psqt_value, Evaluable, ValueScore};
use crate::{
    moves::Move,
    position::{board::Piece, Position},
};

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    let moving_piece = position.board.piece_at(mov.from()).unwrap();

    if mov.flag().is_capture() {
        let captured_piece = position.board.piece_at(mov.to()).unwrap_or(Piece::Pawn);
        score += captured_piece.value() - moving_piece.value() + Piece::Queen.value();
    }

    if mov.flag().is_promotion() {
        let promoted_piece = mov.promotion_piece().unwrap();
        score += promoted_piece.value();
    }

    score += psqt_value(moving_piece, mov.to(), position.side_to_move, 0);
    score -= psqt_value(moving_piece, mov.from(), position.side_to_move, 0);

    score
}

#[cfg(test)]
mod tests {
    use crate::position::{fen::FromFen, Position};

    #[test]
    fn eval_move_heuristic_value() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        let mut moves = position.moves::<false>();
        moves.sort_by(|a, b| {
            super::evaluate_move(&position, *b).cmp(&super::evaluate_move(&position, *a))
        });

        let first_move = moves[0].to_string();
        assert!(first_move == "e2a6" || first_move == "d5e6"); // equal trade of piece
    }
}
