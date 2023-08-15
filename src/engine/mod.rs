use self::commands::{execute_command, parse_command};
use crate::{
    position::{fen::START_FEN, Position},
    search::table::{SearchTable, DEFAULT_TABLE_SIZE_MB},
};
use ahash::AHashMap;
use std::{
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::Duration,
};

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
    Uci,
    Debug(bool),
    IsReady,
    UCINewGame,
    SetOption {
        name: String,
        value: String,
    },

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
    pub game_history: AHashMap<Position, u8>,
    pub stop: Arc<AtomicBool>,
    pub table: Arc<RwLock<SearchTable>>,
}

pub fn uci_loop() {
    let mut engine = Engine {
        position: Position::from_fen(START_FEN).unwrap(),
        stop: Arc::new(AtomicBool::new(true)),
        game_history: AHashMap::new(),
        table: Arc::new(RwLock::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB))),
    };

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if let Ok(command) = parse_command(input) {
            execute_command(command, &mut engine);
        } else {
            println!("Invalid command. Type 'help' to know more.");
        }
    }
}
