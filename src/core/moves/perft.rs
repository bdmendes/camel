use crate::core::position::{MoveStage, Position};

use super::{Move, generate::generate_moves, make::make_move};

pub fn perft<const DIVIDE: bool>(position: &Position, depth: u8) -> (u64, Vec<(Move, u64)>) {
    if depth == 0 {
        return (1, vec![]);
    }

    let moves = generate_moves(position, MoveStage::All);

    if depth == 1 {
        (moves.len() as u64, vec![])
    } else {
        let mut count = 0;
        let mut divided = vec![];
        for m in moves {
            let (branch, _) = perft::<false>(&make_move::<true>(position, m), depth - 1);
            if DIVIDE {
                divided.push((m, branch));
            }
            count += branch;
        }
        (count, divided)
    }
}

#[cfg(test)]
mod tests {
    use super::perft;
    use crate::core::position::{Position, fen::Fen};
    use rstest::rstest;

    #[rstest]
    // Peter Jones Gist (https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9/)
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
    #[case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888)]
    #[case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133)]
    #[case("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467)]
    #[case("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072)]
    #[case("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711)]
    #[case("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206)]
    #[case("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476)]
    #[case("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001)]
    #[case("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658)]
    #[case("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342)]
    #[case("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683)]
    #[case("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217)]
    #[case("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584)]
    #[case("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527)]
    // Chess Programming Wiki (https://www.chessprogramming.org/Perft_Results)
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4865609)]
    #[case(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        4,
        4085603
    )]
    #[case("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 4, 43238)]
    #[case(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        5,
        15833292
    )]
    #[case(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        4,
        2103487
    )]
    #[case(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - -",
        5,
        164075551
    )]
    // Chess960 (https://www.chessprogramming.org/Chess960_Perft_Results)
    #[case(
        "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9",
        5,
        8146062
    )]
    #[case(
        "2nnrbkr/p1qppppp/8/1ppb4/6PP/3PP3/PPP2P2/BQNNRBKR w HEhe - 1 9",
        5,
        16253601
    )]
    #[case(
        "b1q1rrkb/pppppppp/3nn3/8/P7/1PPP4/4PPPP/BQNNRKRB w GE - 1 9",
        5,
        6417013
    )]
    #[case(
        "qbbnnrkr/2pp2pp/p7/1p2pp2/8/P3PP2/1PPP1KPP/QBBNNR1R w hf - 0 9",
        5,
        9183776
    )]
    #[case(
        "1nbbnrkr/p1p1ppp1/3p4/1p3P1p/3Pq2P/8/PPP1P1P1/QNBBNRKR w HFhf - 0 9",
        5,
        34030312
    )]
    #[case(
        "qnbnr1kr/ppp1b1pp/4p3/3p1p2/8/2NPP3/PPP1BPPP/QNB1R1KR w HEhe - 1 9",
        5,
        24851983
    )]
    #[case(
        "q1bnrkr1/ppppp2p/2n2p2/4b1p1/2NP4/8/PPP1PPPP/QNB1RRKB w ge - 1 9",
        5,
        21093346
    )]
    #[case(
        "qbn1brkr/ppp1p1p1/2n4p/3p1p2/P7/6PP/QPPPPP2/1BNNBRKR w HFhf - 0 9",
        5,
        13203304
    )]
    #[case(
        "qnnbbrkr/1p2ppp1/2pp3p/p7/1P5P/2NP4/P1P1PPP1/Q1NBBRKR w HFhf - 0 9",
        5,
        11110203
    )]
    #[case(
        "qn1rbbkr/ppp2p1p/1n1pp1p1/8/3P4/P6P/1PP1PPPK/QNNRBB1R w hd - 2 9",
        5,
        19836606
    )]
    fn perft_test(#[case] fen: Fen, #[case] depth: u8, #[case] nodes: u64) {
        let position = Position::try_from(fen.clone()).unwrap();
        let (count, divided) = perft::<true>(&position, depth);
        for (m, branch) in divided {
            println!("{}: {}", m, branch);
        }
        assert_eq!(count, nodes);
    }
}
