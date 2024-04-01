use camel::{
    position::{fen::FromFen, Position},
    search::{
        constraint::{SearchConstraint, TimeConstraint},
        search_iterative_deepening_multithread,
        table::{SearchTable, DEFAULT_TABLE_SIZE_MB},
        MAX_DEPTH,
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU16},
        Arc,
    },
    time::{Duration, Instant},
};

const SCENARIO_SEARCH_TIME: Duration = Duration::new(30, 0);
const SCENARIO_THREADS: u16 = 2;

fn expect_search(fen: &str, mov: &str) {
    let constraint = SearchConstraint {
        time_constraint: Some(TimeConstraint {
            initial_instant: Instant::now(),
            move_time: SCENARIO_SEARCH_TIME,
        }),
        global_stop: Arc::new(AtomicBool::new(false)),
        threads_stop: Arc::new(AtomicBool::new(false)),
        ponder_mode: Arc::new(AtomicBool::new(false)),
        number_threads: Arc::new(AtomicU16::new(SCENARIO_THREADS)),
        game_history: vec![],
    };
    let table = SearchTable::new(DEFAULT_TABLE_SIZE_MB);

    assert_eq!(
        search_iterative_deepening_multithread(
            &Position::from_fen(fen).unwrap(),
            0,
            MAX_DEPTH,
            Arc::new(table),
            &constraint
        )
        .unwrap()
        .to_string(),
        mov
    );
}

// The following positions are from https://www.chessprogramming.org/Win_at_Chess,
// a collection of tactical positions taken as a sanity check for chess engines.

#[test]
fn wac_1() {
    expect_search("2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - -", "g3g6");
}

#[test]
fn wac_3() {
    expect_search("5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - - 0 1", "e3g3");
}

#[test]
fn wac_4() {
    expect_search("r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - - 0 1", "h6h7");
}

#[test]
fn wac_5() {
    expect_search("5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - - 0 1", "c6c4");
}
