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

const SCENARIO_SEARCH_TIME: Duration = Duration::new(1, 0);
const SCENARIO_THREADS: u16 = 4;

fn expect_search(fen: &str, mov: &str) {
    let table = Arc::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB * SCENARIO_THREADS as usize));

    for cof in 1.. {
        let duration = SCENARIO_SEARCH_TIME * cof;

        if duration > Duration::new(60, 0) {
            panic!("Search failed for {} {}", fen, mov);
        }

        let constraint = SearchConstraint {
            time_constraint: Some(TimeConstraint {
                initial_instant: Instant::now(),
                move_time: duration,
            }),
            global_stop: Arc::new(AtomicBool::new(false)),
            threads_stop: Arc::new(AtomicBool::new(false)),
            ponder_mode: Arc::new(AtomicBool::new(false)),
            number_threads: Arc::new(AtomicU16::new(SCENARIO_THREADS)),
            game_history: vec![],
        };

        let result = search_iterative_deepening_multithread(
            &Position::from_fen(fen).unwrap(),
            0,
            MAX_DEPTH,
            table.clone(),
            &constraint,
        );

        if result.unwrap().to_string() == mov {
            return;
        }
    }

    unreachable!()
}

const WAC_POSITIONS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/books/winatchess.epd"));

#[test]
fn wac() {
    for line in WAC_POSITIONS.lines() {
        let parts = line.split(';').collect::<Vec<_>>();
        let fen = parts[0].split(' ').take(4).collect::<Vec<_>>().join(" ");
        let mov = parts[0].split(' ').last().unwrap();
        let test_number = parts[1]
            .split(' ')
            .last()
            .unwrap()
            .replace('"', "")
            .replace("WAC.", "")
            .parse::<u16>()
            .unwrap();

        println!("WAC {}: {} {}", test_number, fen, mov);
        expect_search(&fen, mov);
        println!("WAC {} passed.\n", test_number);
    }
}
