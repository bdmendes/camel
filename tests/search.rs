use camel::{
    evaluation::{Score, MATE_LOWER, MATE_UPPER},
    position::{moves::Move, Position},
    search::{pvs, search_iterative_deep, Depth, SearchMemo},
};

fn regular_search(position: &Position, depth: Depth) -> (Option<Move>, Score) {
    let now = std::time::Instant::now();
    let (mov, score, nodes) = pvs(
        &position,
        depth,
        MATE_LOWER,
        MATE_UPPER,
        &mut SearchMemo::new(None, None, None),
        depth,
    );
    let elapsed = now.elapsed().as_millis();
    println!(
        "[regular] {}: {} nodes in {} ms at depth {}\n",
        position.to_fen(),
        nodes,
        elapsed,
        depth
    );
    (mov, score)
}

fn iterative_search(position: &Position, depth: Depth) -> (Option<Move>, Score) {
    let now = std::time::Instant::now();
    let (mov, score, nodes) = search_iterative_deep(&position, Some(depth), None, None, None);
    let elapsed = now.elapsed().as_millis();
    println!(
        "[iterative] {}: {} nodes in {} ms at depth {}\n",
        position.to_fen(),
        nodes,
        elapsed,
        depth
    );
    (mov, score)
}

fn test_search(
    fen: &str,
    depth: Depth,
    expected_move: &str,
    expected_lower_score: Option<Score>,
    expected_upper_score: Option<Score>,
) {
    let position = Position::from_fen(fen).unwrap();

    let (reg_mov, reg_score) = iterative_search(&position, depth);
    let (iter_mov, iter_score) = regular_search(&position, depth);

    assert_eq!(reg_mov.unwrap().to_string(), expected_move);
    assert_eq!(iter_mov.unwrap().to_string(), expected_move);

    const MAX_MATE_SCORE_DIFF: Score = 100;
    if let Some(expected_lower_score) = expected_lower_score {
        assert!(reg_score >= expected_lower_score - MAX_MATE_SCORE_DIFF);
        assert!(iter_score >= expected_lower_score - MAX_MATE_SCORE_DIFF);
    }
    if let Some(expected_upper_score) = expected_upper_score {
        assert!(reg_score <= expected_upper_score + MAX_MATE_SCORE_DIFF);
        assert!(iter_score <= expected_upper_score + MAX_MATE_SCORE_DIFF);
    }
}

#[test]
fn search_mate_in_1() {
    test_search("3k4/6R1/8/7R/8/8/8/4k3 w - - 0 1", 5, "h5h8", Some(MATE_UPPER), None);
}

#[test]
fn search_forced_capture() {
    test_search("r2N2k1/p1R3pp/8/1pP5/1b2p1bP/4P3/4BPP1/3K3R b - - 0 24", 4, "a8d8", None, None);
}

#[test]
fn search_trapped_knight() {
    test_search(
        "r2B1rk1/pp1n1pp1/2p1p1p1/8/3P4/5N1P/PPP3P1/R2nR1K1 w - - 0 15",
        4,
        "d8e7",
        None,
        None,
    );
}

#[test]
fn search_check_combination() {
    test_search("4r2k/1p4p1/3P3p/P1P5/4b3/1Bq5/6PP/3QR1K1 b - - 11 41", 4, "c3c5", None, None);
}

#[test]
fn search_endgame_opposition() {
    test_search("5k2/8/6K1/5P2/8/8/8/8 w - - 0 20", 10, "g6f6", None, None);
}
