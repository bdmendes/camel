use crate::{
    moves::Move,
    position::{board::Piece, Position},
};

use super::{piece_value, psqt::psqt_value, ValueScore};

pub fn static_exchange_evaluation(mut position: Position, mut mov: Move) -> ValueScore {
    let mut score = 0;
    let mut found_neutral = false;
    let color = position.side_to_move;

    loop {
        let captured_piece = position.board.piece_at(mov.to()).unwrap_or_else(|| Piece::Pawn);
        let captured_value = piece_value(captured_piece);

        score += if position.side_to_move == color { captured_value } else { -captured_value };

        let new_position = position.make_move(mov);

        if new_position.side_to_move == color && score == 0 {
            found_neutral = true;
        }

        let capturing_moves =
            new_position.moves::<true>().into_iter().filter(|m| m.to() == mov.to());
        let next_move =
            capturing_moves.min_by_key(|m| piece_value(position.board.piece_at(m.from()).unwrap()));

        if let Some(m) = next_move {
            mov = m;
            position = new_position;
        } else {
            break;
        }
    }

    if score < 0 && found_neutral {
        0
    } else {
        score
    }
}

pub fn evaluate_move<const SSE: bool>(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    if mov.flag().is_capture() {
        if SSE {
            score += static_exchange_evaluation(position.clone(), mov);
        } else {
            let captured_piece = position.board.piece_at(mov.to()).unwrap_or_else(|| Piece::Pawn);
            let capturing_piece = position.board.piece_at(mov.from()).unwrap();
            score += piece_value(captured_piece) - piece_value(capturing_piece)
                + piece_value(Piece::Queen);
        }
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
    use crate::{
        evaluation::piece_value,
        position::{board::Piece, Position},
    };

    #[test]
    fn static_exchange_gain() {
        let position =
            Position::from_fen("r5k1/2pp2pp/pnn1p3/2N1P3/8/1B3r1q/PP3P1P/R1BQ1R1K w - - 1 17")
                .unwrap();
        let mov = position.moves::<true>().into_iter().find(|m| m.to_string() == "c5d7").unwrap();

        assert_eq!(super::static_exchange_evaluation(position, mov), piece_value(Piece::Pawn));
    }

    #[test]
    fn static_exchange_neutral() {
        let position = Position::from_fen(
            "r1bq1rk1/1p2bppp/p1n1pn2/3p4/4P3/1NN1B3/PPP1BPPP/R2Q1RK1 w - - 0 10",
        )
        .unwrap();
        let mov = position.moves::<true>().into_iter().find(|m| m.to_string() == "e4d5").unwrap();

        assert_eq!(super::static_exchange_evaluation(position, mov), 0);
    }

    #[test]
    fn static_exchange_loss() {
        let position =
            Position::from_fen("r1bq1rk1/1p2bppp/p1n2n2/3p4/8/1NN1B3/PPP1BPPP/R2Q1RK1 w - - 0 11")
                .unwrap();
        let mov = position.moves::<true>().into_iter().find(|m| m.to_string() == "c3d5").unwrap();

        assert_eq!(
            super::static_exchange_evaluation(position, mov),
            piece_value(Piece::Pawn) - piece_value(Piece::Queen)
        )
    }

    #[test]
    fn eval_move_heuristic_value() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        let mut moves = position.moves::<false>();
        moves.sort_by(|a, b| {
            super::evaluate_move::<false>(&position, *b)
                .cmp(&super::evaluate_move::<false>(&position, *a))
        });

        let first_move = moves[0].to_string();
        assert!(first_move == "e2a6" || first_move == "d5e6"); // equal trade of piece
    }
}
