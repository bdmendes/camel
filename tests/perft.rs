extern crate camel;
use std::collections::HashMap;

use camel::position::{
    movegen::{make_move, Move, MoveGenerator},
    zobrist::ZobristHash,
    Position,
};

fn generate(
    original_depth: u8,
    current_depth: u8,
    position: &Position,
    memo: &mut HashMap<(ZobristHash, u8), (usize, Vec<(Move, usize)>)>,
) -> (usize, Vec<(Move, usize)>) {
    if current_depth == 0 {
        return (1, vec![]);
    }

    let zobrist_hash = position.to_zobrist_hash();
    if let Some((count, moves)) = memo.get(&(zobrist_hash, current_depth)) {
        return (*count, moves.to_vec());
    }

    let moves = MoveGenerator::new().legal_moves(&position, position.to_move);
    let mut res = Vec::with_capacity(moves.len());
    let mut count = 0;

    for move_ in &moves {
        let new_position = make_move(&position, move_);
        let leaf_node_count = generate(original_depth, current_depth - 1, &new_position, memo).0;
        count += leaf_node_count;
        res.push((move_.to_owned(), leaf_node_count));
    }

    memo.insert(
        (zobrist_hash, current_depth),
        (
            count,
            if current_depth == original_depth {
                res.to_vec()
            } else {
                vec![]
            },
        ),
    );

    (count, res)
}

fn perft_divide(fen: &str, depth: u8, expected_nodes: Option<usize>) -> Vec<(Move, usize)> {
    let new_position = || -> Position { Position::from_fen(fen).unwrap() };

    let (count, moves) = generate(depth, depth, &new_position(), &mut HashMap::new());

    if expected_nodes.is_some() {
        assert_eq!(count, expected_nodes.unwrap());
    }

    moves
}

fn perft(fen: &str, depth: u8, expected_nodes: usize) {
    perft_divide(fen, depth, Some(expected_nodes));
}

/* Taken from https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9 */
#[test]
fn gh_perft_1() {
    perft("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8);
}

#[test]
fn gh_perft_2() {
    perft("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8);
}

#[test]
fn gh_perft_3() {
    perft(
        "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2",
        1,
        19,
    );
}

#[test]
fn gh_perft_4() {
    perft(
        "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
        1,
        44,
    );
}

#[test]
fn gh_perft_5() {
    perft(
        "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
        1,
        44,
    );
}

#[test]
fn gh_perft_6() {
    perft(
        "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9",
        1,
        39,
    );
}

#[test]
fn gh_perft_7() {
    perft("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9);
}

#[test]
fn gh_perft_8() {
    perft(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        3,
        62379,
    );
}

#[test]
fn gh_perft_9() {
    perft(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        3,
        89890,
    );
}

#[test]
fn gh_perft_10() {
    perft("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888);
}

#[test]
fn gh_perft_11() {
    perft("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133);
}

#[test]
fn gh_perft_12() {
    perft("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467);
}

#[test]
fn gh_perft_13() {
    perft("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072);
}

#[test]
fn gh_perft_14() {
    perft("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711);
}

#[test]
fn gh_perft_15() {
    perft("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206);
}

#[test]
fn gh_perft_16() {
    perft("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476);
}

#[test]
fn gh_perft_17() {
    perft("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001);
}

#[test]
fn gh_perft_18() {
    perft("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658);
}

#[test]
fn gh_perft_19() {
    perft("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342);
}

#[test]
fn gh_perft_20() {
    perft("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683);
}

#[test]
fn gh_perft_21() {
    perft("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217);
}

#[test]
fn gh_perft_22() {
    perft("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584);
}

#[test]
fn gh_perft_23() {
    perft("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527);
}

/* Expected divides taken from Stockfish */
#[test]
fn perft_kiwipete() {
    let kiwipete_test_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    let expected_divides = [
        ("a2a3", 2186),
        ("b2b3", 1964),
        ("g2g3", 1882),
        ("d5d6", 1991),
        ("a2a4", 2149),
        ("g2g4", 1843),
        ("g2h3", 1970),
        ("d5e6", 2241),
        ("c3b1", 2038),
        ("c3d1", 2040),
        ("c3a4", 2203),
        ("c3b5", 2138),
        ("e5d3", 1803),
        ("e5c4", 1880),
        ("e5g4", 1878),
        ("e5c6", 2027),
        ("e5g6", 1997),
        ("e5d7", 2124),
        ("e5f7", 2080),
        ("d2c1", 1963),
        ("d2e3", 2136),
        ("d2f4", 2000),
        ("d2g5", 2134),
        ("d2h6", 2019),
        ("e2d1", 1733),
        ("e2f1", 2060),
        ("e2d3", 2050),
        ("e2c4", 2082),
        ("e2b5", 2057),
        ("e2a6", 1907),
        ("a1b1", 1969),
        ("a1c1", 1968),
        ("a1d1", 1885),
        ("h1f1", 1929),
        ("h1g1", 2013),
        ("f3d3", 2005),
        ("f3e3", 2174),
        ("f3g3", 2214),
        ("f3h3", 2360),
        ("f3f4", 2132),
        ("f3g4", 2169),
        ("f3f5", 2396),
        ("f3h5", 2267),
        ("f3f6", 2111),
        ("e1d1", 1894),
        ("e1f1", 1855),
        ("e1g1", 2059),
        ("e1c1", 1887),
    ];

    let moves = perft_divide(kiwipete_test_fen, 3, None);
    for (mv, count) in moves {
        if expected_divides.contains(&(&mv.to_string(), count - 1)) {
            continue;
        }
        println!("Unexpected divide: {} {}", mv, count);
    }
}
