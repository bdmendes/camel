use crate::core::{Position, color::Color, piece::Piece, square::Square};

use super::Move;

fn see_recurse(
    square: Square,
    position: &mut Position,
    side_to_move: Color,
    at_square: Piece,
) -> i8 {
    let attackers = position.attackers(square, side_to_move);
    if attackers.is_empty() {
        return 0;
    }

    let (least_square, least_piece) = attackers
        .into_iter()
        .map(|sq| (sq, position.piece_at(sq).unwrap()))
        .min_by(|a, b| a.1.value().cmp(&b.1.value()))
        .unwrap();

    position.clear_square_low::<false>(least_square);

    let op_see = see_recurse(square, position, side_to_move.flipped(), least_piece);
    std::cmp::max(0, at_square.value() - op_see)
}

pub fn see(mov: Move, position: &Position) -> i8 {
    let mut position = *position;
    let side_to_move = position.side_to_move.flipped();

    let from_square = mov.from();
    let from_piece = position.piece_at(from_square).unwrap();
    let to_square = mov.to();
    let to_piece = position.piece_at(to_square).unwrap_or(Piece::Pawn);

    position.clear_square_low::<false>(from_square);

    to_piece.value() - see_recurse(to_square, &mut position, side_to_move, from_piece)
}

#[cfg(test)]
mod tests {
    use crate::core::{MoveStage, Position, moves::see};
    use std::str::FromStr;

    fn assert_see(position: &str, mov: &str, value: i8) {
        let position = Position::from_str(position).unwrap();
        let mov = *position
            .moves(MoveStage::CapturesAndPromotions)
            .iter()
            .find(|m| m.to_string().as_str() == mov)
            .unwrap();
        assert_eq!(see::see(mov, &position), value);
    }

    #[test]
    fn free_piece_instant() {
        assert_see(
            "r2qk1nr/pp3ppp/2nBp3/3p4/3P2b1/5N2/PPP1BPPP/RN1Q1RK1 b kq - 0 8",
            "d8d6",
            3,
        );
    }

    #[test]
    fn free_piece_instant_with_kings() {
        assert_see(
            "8/1p3p2/1P2p1k1/pP1pP1p1/3P1pKP/5P2/6P1/8 w - - 1 38",
            "h4g5",
            1,
        );
    }

    #[test]
    fn free_piece_after_exchange() {
        assert_see(
            "2r1r1k1/pp4pp/2n1qnp1/3p2P1/7P/2P2N2/PP2BP2/2RQ1RK1 b - - 0 19",
            "e6e2",
            3,
        );
    }

    #[test]
    fn equal_exchange() {
        assert_see(
            "r3k1nr/pp3ppp/3qp3/3p4/3n2b1/P4N1P/1PP1BPP1/RN1Q1RK1 b kq - 0 10",
            "d4f3",
            0,
        );
    }

    #[test]
    fn equal_exchange_ep() {
        assert_see(
            "r1bqkbnr/ppp1pppp/2n5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
            "e5d6",
            0,
        );
    }
}
