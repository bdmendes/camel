use crate::engine::{time::get_duration, Engine};
use camel::{
    evaluation::Evaluable,
    moves::gen::{perft, MoveStage},
    position::{
        fen::{FromFen, ToFen, START_FEN},
        Position,
    },
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

    let mut constraint = SearchConstraint {
        branch_history: engine.game_history.clone(),
        time_constraint: calc_move_time
            .map(|t| TimeConstraint { initial_instant: std::time::Instant::now(), move_time: t }),
        stop_now: Some(stop_now.clone()),
        ponder_mode: Some(engine.pondering.clone()),
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
    println!("id name Camel");
    println!("id author Bruno Mendes");

    // Options list
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
            engine.table.lock().unwrap().set_size(size.clamp(MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB));
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
    engine.table.lock().unwrap().clear();
}

pub fn execute_perft(depth: u8, position: &Position) {
    println!("Perft will run in the background and report results when done.");

    let position = *position;

    thread::spawn(move || {
        let start = std::time::Instant::now();
        let nodes = perft::<false>(&position, depth);
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
