use camel::{
    moves::{
        gen::{generate_moves, MoveStage},
        make_move,
    },
    position::{board::ZobristHash, Position},
    search::Depth,
};
use std::collections::HashMap;

fn perft<const ROOT: bool, const BULK_AT_HORIZON: bool, const HASH: bool>(
    position: &Position,
    depth: u8,
    cache: &mut HashMap<(ZobristHash, Depth), u64>,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    if HASH {
        if let Some(res) = cache.get(&(position.zobrist_hash(), depth)) {
            return *res;
        }
    }

    let moves = generate_moves(MoveStage::All, position);

    if BULK_AT_HORIZON && depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;

    for mov in moves {
        let new_position = make_move::<true>(position, mov);
        let count = perft::<false, BULK_AT_HORIZON, HASH>(&new_position, depth - 1, cache);
        nodes += count;
    }

    if HASH {
        cache.insert((position.zobrist_hash(), depth), nodes);
    }

    nodes
}

fn expect_perft(fen: &str, depth: u8, nodes: u64) {
    let position = Position::from_fen(fen).unwrap();
    let res = perft::<true, true, false>(&position, depth, &mut HashMap::new());
    assert_eq!(res, nodes);
}

#[test]
fn perft_gh_1() {
    expect_perft("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8);
}

#[test]
fn perft_gh_2() {
    expect_perft("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8);
}

#[test]
fn perft_gh_3() {
    expect_perft("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2", 1, 19);
}

#[test]
fn perft_gh_4() {
    expect_perft("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2", 1, 44);
}

#[test]
fn perft_gh_5() {
    expect_perft("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2", 1, 44);
}

#[test]
fn perft_gh_6() {
    expect_perft("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9", 1, 39);
}

#[test]
fn perft_gh_7() {
    expect_perft("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9);
}

#[test]
fn perft_gh_8() {
    expect_perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379);
}

#[test]
fn perft_gh_9() {
    expect_perft(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        3,
        89890,
    );
}

#[test]
fn perft_gh_10() {
    expect_perft("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888);
}

#[test]
fn perft_gh_11() {
    expect_perft("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133);
}

#[test]
fn perft_gh_12() {
    expect_perft("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467);
}

#[test]
fn perft_gh_13() {
    expect_perft("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072);
}

#[test]
fn perft_gh_14() {
    expect_perft("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711);
}

#[test]
fn perft_gh_15() {
    expect_perft("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206);
}

#[test]
fn perft_gh_16() {
    expect_perft("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476);
}

#[test]
fn perft_gh_17() {
    expect_perft("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001);
}

#[test]
fn perft_gh_18() {
    expect_perft("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658);
}

#[test]
fn perft_gh_19() {
    expect_perft("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342);
}

#[test]
fn perft_gh_20() {
    expect_perft("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683);
}

#[test]
fn perft_gh_21() {
    expect_perft("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217);
}

#[test]
fn perft_gh_22() {
    expect_perft("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584);
}

#[test]
fn perft_gh_23() {
    expect_perft("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527);
}
