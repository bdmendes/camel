use crate::position::{Position, color::Color, piece::Piece, square::Square};

use super::Move;

fn see_recurse(square: Square, position: &mut Position, side_to_move: Color, at_square: Piece) -> i8 {
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
    let side_to_move = position.side_to_move().flipped();

    let from_square = mov.from();
    let from_piece = position.piece_at(from_square).unwrap();
    let to_square = mov.to();
    let to_piece = position.piece_at(to_square).unwrap_or(Piece::Pawn);

    position.clear_square_low::<false>(from_square);

    to_piece.value() - see_recurse(to_square, &mut position, side_to_move, from_piece)
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::see,
        position::{MoveStage, Position, fen::Fen},
    };
    use rstest::rstest;

    #[rstest]
    #[case("r2qk1nr/pp3ppp/2nBp3/3p4/3P2b1/5N2/PPP1BPPP/RN1Q1RK1 b kq - 0 8", "d8d6", 3)]
    #[case("r2qk1nr/pp3ppp/2nBp3/3p4/3P2b1/5N2/PPP1BPPP/RN1Q1RK1 b kq - 0 8", "g4f3", 0)]
    #[case("8/1p3p2/1P2p1k1/pP1pP1p1/3P1pKP/5P2/6P1/8 w - - 1 38", "h4g5", 1)]
    #[case("2r1r1k1/pp4pp/2n1qnp1/3p2P1/7P/2P2N2/PP2BP2/2RQ1RK1 b - - 0 19", "e6e2", 3)]
    #[case("r3k1nr/pp3ppp/3qp3/3p4/3n2b1/P4N1P/1PP1BPP1/RN1Q1RK1 b kq - 0 10", "d4f3", 0)]
    #[case("r3k1nr/pp3ppp/3qp3/3p4/3n2b1/P4N1P/1PP1BPP1/RN1Q1RK1 b kq - 0 10", "d4c2", -2)]
    #[case("r1bqkbnr/ppp1pppp/2n5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3", "e5d6", 0)]
    #[case("rnbq1rk1/pp3ppp/2p5/3p4/1bPPn3/2N2N2/PPQ1BPPP/R1B1K2R w KQ - 0 10", "c4d5", 0)]
    #[case("r1bq1rk1/pp3ppp/5n2/2pp4/1bPPn3/1QN1BN2/PP2BPPP/R4RK1 w - - 0 13", "c3d5", 1)]
    #[case("r3r1k1/pp1q2pp/5p2/3NP3/4Q3/P7/1P3PPP/3R1RK1 b - - 1 21", "d7d5", -6)]
    #[case("3rr1k1/pp3qpp/5p2/3NP3/4Q3/P7/1P1R1PPP/3R2K1 b - - 5 23", "d8d5", -2)]
    #[case("3r3r/pp1n1kpp/1qpb1p2/3b1Q2/3P4/2N2N2/PP3PPP/R3R1K1 w - - 2 17", "c3d5", 1)]
    #[case("1n1r1k1r/1p4pp/pq1bRp2/5Q2/3P4/5N2/PP3PPP/R5K1 w - - 0 21", "e6f6", -3)]
    #[case("r4rk1/pp3ppp/1q2p1n1/1BbpP3/3n1P2/2N1B3/PP3QPP/R4R1K w - - 2 15", "e3d4", 0)]
    #[case("2r1r1k1/1p2bppp/p3p1n1/q2pPP2/3P2P1/2NBRQ2/PP5P/5R1K b - - 0 19", "c8c3", -1)]
    fn see_sequence(#[case] fen: Fen, #[case] mov: &str, #[case] value: i8) {
        let position = Position::try_from(fen).unwrap();
        let mov = *position
            .moves(MoveStage::CapturesAndPromotions)
            .iter()
            .find(|m| m.to_string() == mov)
            .unwrap();
        assert_eq!(see::see(mov, &position), value);
    }
}
