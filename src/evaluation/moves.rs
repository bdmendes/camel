use crate::{
    moves::Move,
    position::{board::Piece, Position},
};

use super::{piece_value, psqt::psqt_value, ValueScore};

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    if mov.flag().is_capture() {
        let captured_piece = position.board.piece_at(mov.to()).unwrap_or(Piece::Pawn);
        let capturing_piece = position.board.piece_at(mov.from()).unwrap();
        score +=
            piece_value(captured_piece) - piece_value(capturing_piece) + piece_value(Piece::Queen);
    }

    if mov.flag().is_promotion() {
        let promoted_piece = mov.promotion_piece().unwrap();
        score += piece_value(promoted_piece);
    }

    let piece = position.board.piece_at(mov.from()).unwrap();
    score += psqt_value(piece, mov.to(), position.side_to_move, 0);
    score -= psqt_value(piece, mov.from(), position.side_to_move, 0);

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
        let mut moves = position.moves::<false>();
        moves.sort_by(|a, b| {
            super::evaluate_move(&position, *b).cmp(&super::evaluate_move(&position, *a))
        });

        let first_move = moves[0].to_string();
        assert!(first_move == "e2a6" || first_move == "d5e6"); // equal trade of piece
    }
}
