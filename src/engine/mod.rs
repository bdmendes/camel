use std::{
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::Duration,
};

use camel::{
    position::{fen::START_FEN, Position},
    search::table::SearchTable,
};
use once_cell::sync::Lazy;

use self::commands::{execute_command, parse_command};

mod commands;
mod time;

pub enum Command {
    // Standard UCI commands
    Position {
        position: Position,
        game_history: Vec<Position>,
    },
    Go {
        depth: Option<u8>,
        move_time: Option<Duration>,
        white_time: Option<Duration>,
        black_time: Option<Duration>,
        white_increment: Option<Duration>,
        black_increment: Option<Duration>,
    },
    Stop,
    UCI,
    Debug(bool),
    IsReady,
    UCINewGame,

    // Custom commands
    Perft {
        depth: u8,
    },
    DoMove {
        mov_str: String,
    },
    Display,
    AllMoves,
    Help,
    Clear,
    Quit,
}

pub struct Engine {
    pub position: Position,
    pub game_history: Vec<Position>,
    pub stop: Arc<AtomicBool>,
    pub table: Arc<RwLock<SearchTable>>,
}

pub fn uci_loop() {
    static mut ENGINE: Lazy<Engine> = Lazy::new(|| Engine {
        position: Position::from_fen(START_FEN).unwrap(),
        stop: Arc::new(AtomicBool::new(true)),
        game_history: Vec::new(),
        table: Arc::new(RwLock::new(SearchTable::new())),
    });

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if let Ok(command) = parse_command(input) {
            unsafe {
                execute_command(command, &mut ENGINE);
            }
        } else {
            println!("Invalid command. Type 'help' to know more.");
        }
    }
}
