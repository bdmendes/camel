use crate::engine::{time::get_duration, Engine, DEFAULT_NUMBER_THREADS, MAX_THREADS};
use camel::{
    evaluation::{Evaluable, ValueScore},
    moves::gen::{perft, MoveStage},
    position::{
        fen::{FromFen, ToFen, START_FEN},
        Color, Position,
    },
    search::{
        constraint::{SearchConstraint, TimeConstraint},
        history::HistoryEntry,
        pvs::quiesce,
        search_iterative_deepening_multithread,
        table::{DEFAULT_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB},
        Depth, MAX_DEPTH,
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

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
    players_time: (Option<Duration>, Option<Duration>),
    players_increment: (Option<Duration>, Option<Duration>),
    ponder: bool,
) {
    if !engine.stop.load(Ordering::Relaxed) {
        return;
    }

    engine.pondering.store(ponder, Ordering::Relaxed);

    let position = engine.position;

    let mut white_time = players_time.0;
    let mut black_time = players_time.1;

    if white_time.is_some() && black_time.is_none() {
        black_time = white_time;
    } else if black_time.is_some() && white_time.is_none() {
        white_time = black_time;
    }

    let white_increment = players_increment.0;
    let black_increment = players_increment.1;

    let calc_move_time = match move_time {
        Some(t) => Some(t),
        None if white_time.is_some() => Some(get_duration(
            &position,
            white_time.unwrap(),
            black_time.unwrap(),
            white_increment,
            black_increment,
            ponder,
        )),
        None => None,
    };

    let stop_now = engine.stop.clone();
    let table = engine.table.clone();

    let constraint = SearchConstraint {
        game_history: engine.game_history.clone(),
        time_constraint: calc_move_time
            .map(|t| TimeConstraint { initial_instant: std::time::Instant::now(), move_time: t }),
        global_stop: stop_now.clone(),
        threads_stop: Arc::new(AtomicBool::new(false)),
        ponder_mode: engine.pondering.clone(),
        number_threads: engine.number_threads.clone(),
    };

    thread::spawn(move || {
        stop_now.store(false, Ordering::Relaxed);
        let current_guess = quiesce(&position, ValueScore::MIN + 1, ValueScore::MAX, &constraint).0;
        search_iterative_deepening_multithread(
            &position,
            current_guess,
            depth.map_or_else(|| MAX_DEPTH, |d| d as Depth),
            table.clone(),
            &constraint,
        );
        stop_now.store(true, Ordering::Relaxed);
    });
}

pub fn execute_stop(engine: &mut Engine) {
    if engine.stop.load(Ordering::Relaxed) {
        return;
    }
    engine.pondering.store(false, Ordering::Relaxed);
    engine.stop.store(true, Ordering::Relaxed);
}

pub fn execute_ponderhit(engine: &mut Engine) {
    if !engine.pondering.load(Ordering::Relaxed) {
        return;
    }
    engine.pondering.store(false, Ordering::Relaxed);
}

pub fn execute_uci() {
    println!("id name Camel {}", env!("CARGO_PKG_VERSION"));
    println!("id author Bruno Mendes");

    println!(
        "option name Threads type spin default {} min 1 max {}",
        DEFAULT_NUMBER_THREADS, MAX_THREADS
    );
    println!(
        "option name Hash type spin default {} min {} max {}",
        DEFAULT_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB
    );
    println!("option name UCI_Chess960 type check default true",);
    println!("option name Ponder type check default true",);

    println!("uciok");
}

pub fn execute_is_ready() {
    println!("readyok");
}

pub fn execute_debug(_: bool) {}

pub fn execute_set_option(name: &str, value: &str, engine: &mut Engine) {
    if name == "Hash" {
        if let Ok(size) = value.parse::<usize>() {
            engine.table.set_size(size.clamp(MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB));
        }
    } else if name == "Threads" {
        if let Ok(threads) = value.parse::<u16>() {
            engine.number_threads.store(threads.clamp(1, MAX_THREADS), Ordering::Relaxed);
        }
    } else if name == "Ponder" || name == "UCI_Chess960" {
        // The time management bonus already takes pondering into account, so do nothing.
        // The engine is compliant with Chess 960 by design, so do nothing.
    } else {
        println!("Option not supported: {}", name);
    }
}

pub fn execute_uci_new_game(engine: &mut Engine) {
    engine.position = Position::from_fen(START_FEN).unwrap();
    engine.game_history = Vec::new();
    engine.table.clear();
}

pub fn execute_perft(depth: u8, position: &Position) {
    println!("Perft will run in the background and report results when done.");

    let position = *position;

    thread::spawn(move || {
        let start = std::time::Instant::now();
        let nodes = perft::<false, true>(&position, depth);
        let elapsed = start.elapsed();

        println!("Perft results for depth {}", depth);
        println!("-> Nodes: {}", nodes);
        println!("-> Time: {}s", elapsed.as_secs_f32());
        println!("-> Mnps: {}", nodes as f64 / elapsed.as_secs_f64() / 1000000.0);
    });
}

pub fn execute_do_move(mov_str: &str, position: &mut Position) {
    if let Some(mov) = position.moves(MoveStage::All).iter().find(|mov| mov.to_string() == mov_str)
    {
        *position = position.make_move(*mov);
    } else {
        println!("Illegal move: {}", mov_str);
    }
}

pub fn execute_display(position: &Position) {
    print!("{}", position.board);
    println!("{}", position.to_fen());
    println!("Static evaluation: {}", position.value());
    println!("Chess960: {}", position.is_chess960);
    println!(
        "{} to play.",
        match position.side_to_move {
            Color::White => "White",
            Color::Black => "Black",
        }
    );
}

pub fn execute_all_moves(position: &Position) {
    let moves = position.moves(MoveStage::All);
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
    println!("   'perft <depth>': run perft on the current position with the given depth");
    println!("   'move <move>': perform given move in uci notation on the current board");
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
