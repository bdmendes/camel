use crate::{
    moves::Move,
    position::{Position, color::Color, piece::Piece},
};

const MAX_SEE_DEPTH: usize = 8;

pub fn static_exchange(position: &Position, mov: Move) -> i8 {
    let mut position = *position;
    let square = mov.to();

    let mut piece = mov
        .promotion_piece()
        .unwrap_or_else(|| position.piece_at(mov.from()).unwrap());
    let mut gains = [0i8; MAX_SEE_DEPTH];
    gains[0] =
        position.piece_at(square).unwrap_or(Piece::Pawn).value() + mov.promotion_piece().map_or(0, |p| p.value() - 1);
    let mut side_to_move = position.side_to_move();
    let mut depth = 0;

    position.clear_square_low::<false>(mov.from());

    loop {
        depth += 1;
        side_to_move = side_to_move.flipped();
        if let Some(attacker) = position
            .attackers(square, side_to_move)
            .min_by_key(|&sq| position.piece_at(sq).unwrap().value())
        {
            gains[depth] += piece.value() - gains[depth - 1];
            piece = position.piece_at(attacker).unwrap();
            if piece == Piece::Pawn
                && (side_to_move == Color::White && attacker.rank() == 6
                    || side_to_move == Color::Black && attacker.rank() == 1)
            {
                gains[depth] += Piece::Queen.value() - 1;
            }
            position.clear_square_low::<false>(attacker);
        } else {
            break;
        }
        if depth == MAX_SEE_DEPTH - 1 {
            break;
        }
    }

    loop {
        depth -= 1;
        if depth == 0 {
            break gains[0];
        }
        gains[depth - 1] = -(gains[depth].max(-gains[depth - 1]));
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        position::{MoveStage, Position, fen::Fen},
        search::see::static_exchange,
    };

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
    #[case("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -", "e1e5", 1)]
    #[case("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -", "d3e5", -2)]
    #[case("r4R2/1P6/6K1/4k3/8/8/8/r7 w - - 0 1", "b7a8q", 9)]
    #[case("Q7/1P5k/1n6/8/8/4K3/8/8 b - - 0 1", "b6a8", -2)]
    fn see_sequence(#[case] fen: Fen, #[case] mov: &str, #[case] value: i8) {
        let position = Position::try_from(fen).unwrap();
        let mov = *position
            .moves(MoveStage::CapturesAndPromotions)
            .iter()
            .find(|m| m.to_string() == mov)
            .unwrap();
        assert_eq!(static_exchange(&position, mov), value);
    }
}
