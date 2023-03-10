use crate::position::{
    moves::{Move, MoveFlags},
    Piece, Position,
};

use super::{piece_value, psqt::psqt_value, Score, MATE_UPPER};

pub fn evaluate_move(
    move_: &Move,
    position: &Position,
    killer_move: bool,
    hash_move: bool,
) -> Score {
    if hash_move {
        return MATE_UPPER;
    }

    let mut score: Score = 0;

    if killer_move {
        score += piece_value(Piece::WQ); // better than quiet moves and bad captures
    }

    if move_.promotion.is_some() {
        score += piece_value(move_.promotion.unwrap());
    }

    let moved_piece = position.board[move_.from].unwrap();

    if move_.flags.contains(MoveFlags::CAPTURE) {
        let moved_piece_value = piece_value(moved_piece);
        let captured_piece_value = piece_value(position.board[move_.to].unwrap());
        score = captured_piece_value - moved_piece_value + piece_value(Piece::WQ);
    }

    let start_psqt_value = psqt_value(moved_piece, move_.from, 0);
    let end_psqt_value = psqt_value(moved_piece, move_.to, 0);
    score += end_psqt_value - start_psqt_value;

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
        let first_move = moves[0].to_string();
        assert!(first_move == "e2a6" || first_move == "d5e6"); // equal trade of piece
    }
}
