use crate::core::{MoveStage, Position};

use super::{gen::generate_moves, make::make_move};

pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(position, MoveStage::All);

    if depth == 1 {
        moves.len() as u64
    } else {
        moves
            .iter()
            .map(|mov| perft(&make_move::<true>(position, *mov), depth - 1))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{fen::Fen, moves::perft::perft, Position};
    use rstest::rstest;

    #[rstest]
    #[case("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8)]
    #[case("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8)]
    #[case("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2", 1, 19)]
    #[case(
        "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2",
        1,
        5
    )]
    #[case(
        "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
        1,
        44
    )]
    #[case("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9", 1, 39)]
    #[case("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9)]
    #[case("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379)]
    #[case(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        3,
        89890
    )]
    fn peter_jones_gist(#[case] fen: Fen, #[case] depth: u8, #[case] nodes: u64) {
        let position = Position::try_from(fen.clone()).unwrap();
        assert_eq!(perft(&position, depth), nodes);
    }
}
