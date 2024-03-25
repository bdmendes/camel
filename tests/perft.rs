use camel::position::fen::FromFen;
use camel::{moves::gen::perft, position::Position};

fn expect_perft(fen: &str, depth: u8, nodes: u64) {
    let position = Position::from_fen(fen).unwrap();
    assert_eq!(perft::<false, true>(&position, depth), nodes);
    assert_eq!(perft::<true, true>(&position, depth), nodes);
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

#[test]
fn perft_chess960_1() {
    expect_perft("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9", 5, 8146062);
}

#[test]
fn perft_chess960_2() {
    expect_perft("nnr1kqbr/pp1pp1p1/2p5/b4p1p/P7/1PNP4/2P1PPPP/N1RBKQBR w HChc - 1 9", 5, 4266410);
}

#[test]
fn perft_chess960_3() {
    expect_perft("1rqkbnrb/pp1ppp1p/1n4p1/B1p5/3PP3/4N3/PPP2PPP/NRQK2RB w GBgb - 0 9", 5, 19715083);
}

#[test]
fn perft_chess960_4() {
    expect_perft("nrkq1rbb/pp1ppp1p/2pn4/8/PP3Pp1/7P/2PPP1P1/NRKQNRBB w FBfb - 0 9", 5, 19867117)
}

#[test]
fn perft_chess960_5() {
    expect_perft("rbqnbknr/pp1pppp1/8/2p5/3P3p/5N1P/PPP1PPPR/RBQNBK2 w Aha - 0 9", 5, 26363334);
}

#[test]
fn perft_chess960_6() {
    expect_perft("rbnnkr1q/1ppp2pp/p4p2/P2bp3/4P2P/8/1PPP1PP1/RBNNKRBQ w FAfa - 1 9", 5, 21591790);
}

#[test]
fn perft_chess960_7() {
    expect_perft(
        "brqkr1nb/2ppp1pp/1p2np2/p7/2P1PN2/8/PP1P1PPP/BRQKRN1B w EBeb - 0 9 	",
        5,
        15516491,
    );
}

#[test]
fn perft_chess960_8() {
    expect_perft("rkbbqr1n/1p1pppp1/2p2n2/p4NBp/8/3P4/PPP1PPPP/RK1BQRN1 w FAfa - 0 9", 5, 26676373);
}

#[test]
fn perft_chess960_9() {
    expect_perft("rkr1nbbq/2ppp1pp/1pn5/p4p2/P6P/3P4/1PP1PPPB/RKRNNB1Q w CAca - 1 9", 5, 11484012);
}

#[test]
fn perft_chess960_10() {
    expect_perft("bbq1nr1r/pppppk1p/2n2p2/6p1/P4P2/4P1P1/1PPP3P/BBQNNRKR w HF - 1 9", 5, 10316716);
}
