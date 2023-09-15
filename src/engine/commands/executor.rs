use crate::engine::{time::get_duration, Engine};
use camel::{
    evaluation::position::evaluate_position,
    moves::gen::perft,
    position::{fen::START_FEN, Position},
    search::{
        constraint::{HistoryEntry, SearchConstraint, TimeConstraint},
        search_iter,
        table::{DEFAULT_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB},
        Depth, MAX_DEPTH,
    },
};
use std::{sync::atomic::Ordering, thread, time::Duration};

pub fn execute_position(new_position: &Position, game_history: &[Position], engine: &mut Engine) {
    engine.position = *new_position;
    engine.game_history = game_history
        .iter()
        .map(|position| HistoryEntry { hash: position.zobrist_hash(), reversible: true })
        .collect();
}

pub fn execute_go(
    engine: &mut Engine,
    depth: Option<u8>,
    move_time: Option<Duration>,
    mut white_time: Option<Duration>,
    mut black_time: Option<Duration>,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
) {
    if !engine.stop.load(Ordering::Relaxed) {
        return;
    }
    let position = engine.position;

    if white_time.is_some() && black_time.is_none() {
        black_time = white_time;
    } else if black_time.is_some() && white_time.is_none() {
        white_time = black_time;
    }

    let calc_move_time = match move_time {
        Some(t) => Some(t),
        None if white_time.is_some() => Some(get_duration(
            &position,
            white_time.unwrap(),
            black_time.unwrap(),
            white_increment,
            black_increment,
        )),
        None => None,
    };

    let stop_now = engine.stop.clone();
    let table = engine.table.clone();

    let mut constraint = SearchConstraint {
        branch_history: engine.game_history.clone(),
        time_constraint: calc_move_time
            .map(|t| TimeConstraint { initial_instant: std::time::Instant::now(), move_time: t }),
        stop_now: Some(stop_now.clone()),
    };

    thread::spawn(move || {
        stop_now.store(false, Ordering::Relaxed);
        search_iter(
            &position,
            depth.map_or_else(|| MAX_DEPTH, |d| d as Depth),
            table.clone(),
            &mut constraint,
        );
        stop_now.store(true, Ordering::Relaxed);
    });
}

pub fn execute_stop(engine: &mut Engine) {
    if engine.stop.load(Ordering::Relaxed) {
        return;
    }
    engine.stop.store(true, Ordering::Relaxed);
}

pub fn execute_uci() {
    println!("id name Camel");
    println!("id author Bruno Mendes");

    println!(
        "option name Hash type spin default {} min {} max {}",
        DEFAULT_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB
    );

    println!("uciok");
}

pub fn execute_is_ready() {
    println!("readyok");
}

pub fn execute_debug(_: bool) {}

pub fn execute_set_option(name: &str, value: &str, engine: &mut Engine) {
    if name == "Hash" {
        if let Ok(size) = value.parse::<usize>() {
            engine
                .table
                .lock()
                .unwrap()
                .set_size(size.min(MAX_TABLE_SIZE_MB).max(MIN_TABLE_SIZE_MB));
        }
    }
}

pub fn execute_uci_new_game(engine: &mut Engine) {
    engine.position = Position::from_fen(START_FEN).unwrap();
    engine.game_history = Vec::new();
    engine.table.lock().unwrap().clear();
}

pub fn execute_perft(depth: u8, position: &Position) {
    let position = *position;
    thread::spawn(move || {
        let time = std::time::Instant::now();
        let (nodes, _) = perft::<true, true, false>(&position, depth);
        let elapsed = time.elapsed().as_millis();
        let mnps = nodes as f64 / 1000.0 / (elapsed + 1) as f64;
        println!("Searched {} nodes in {} ms [{:.3} Mnps]", nodes, elapsed, mnps);
    });
}

pub fn execute_do_move(mov_str: &str, position: &mut Position) {
    if let Some(mov) = position.moves(false).iter().find(|mov| mov.to_string() == mov_str) {
        *position = position.make_move(*mov);
    } else {
        println!("Illegal move: {}", mov_str);
    }
}

pub fn execute_display(position: &Position) {
    print!("{}", position.board);
    println!("{}", position.to_fen());
    println!("Static evaluation: {}", evaluate_position(position));
}

pub fn execute_all_moves(position: &Position) {
    let moves = position.moves(false);
    for mov in moves {
        print!("{} ", mov);
    }
    println!();
}

pub fn execute_help() {
    println!("================================================================================");
    println!("Camel is a UCI-compatible chess engine, primarily meant to be used inside a GUI.");
    println!("You can review the UCI standard in https://backscattering.de/chess/uci/.");
    println!("Camel also bundles support for custom commands, for debugging purposes:");
    println!("   'perft <depth>': count nodes searched from current position until given depth");
    println!("   'domove <move>': perform given move in uci notation on the current board");
    println!("   'list': list legal moves available on the current position");
    println!("   'display': print current position");
    println!("   'help': print this help message");
    println!("   'clear': clear the screen");
    println!("   'quit': exit the program");
    println!("For more information, please visit https://github.com/bdmendes/camel/.");
    println!("================================================================================");
}

pub fn execute_clear() {
    if !std::process::Command::new("clear").status().unwrap().success() {
        std::process::Command::new("cls");
    }
}

pub fn execute_quit() {
    std::process::exit(0);
}
