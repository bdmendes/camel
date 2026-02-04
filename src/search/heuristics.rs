use crate::position::{Position, piece::Piece};

pub fn maybe_zug(position: &Position) -> bool {
    let occ = position.occupancy_bb_all() & !position.pieces_bb(Piece::King);
    occ.count_ones() <= 2 || (occ & !position.pieces_bb(Piece::Pawn)).is_empty()
}

#[cfg(test)]
mod tests {
    use crate::{
        position::{Position, fen::Fen},
        search::heuristics::maybe_zug,
    };
    use rstest::rstest;

    #[rstest]
    #[case("4k3/8/1p3P2/5K2/8/8/8/8 w - - 0 1", true)]
    #[case("8/8/3k4/8/6r1/8/3K4/8 w - - 0 1", true)]
    #[case("8/8/2PkP3/3P4/3K4/8/8/8 w - - 0 1", true)]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", false)]
    #[case("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", false)]
    fn zug(#[case] fen: Fen, #[case] res: bool) {
        let position = Position::try_from(fen).unwrap();
        assert_eq!(maybe_zug(&position), res);
    }
}
