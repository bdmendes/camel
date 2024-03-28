use self::commands::{execute_command, parse_command};
use camel::{
    position::{
        fen::{FromFen, START_FEN},
        Position,
    },
    search::{
        history::HistoryEntry,
        table::{SearchTable, DEFAULT_TABLE_SIZE_MB},
    },
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU16},
        Arc, RwLock,
    },
    time::Duration,
};

mod commands;
mod time;

pub const DEFAULT_NUMBER_THREADS: u16 = 1;
pub const MAX_THREADS: u16 = 1024;

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
        ponder: bool,
    },
    Stop,
    PonderHit,
    Uci,
    Debug(bool),
    IsReady,
    UCINewGame,
    SetOption {
        name: String,
        value: String,
    },

    // Custom commands
    AutoMove {
        seconds: u16,
    },
    Perft(u8),
    DoMove {
        mov_str: String,
    },
    Display,
    ListMoves,
    Help,
    Clear,
    Quit,
}

pub struct Engine {
    pub position: Position,
    pub game_history: Vec<HistoryEntry>,
    pub table: Arc<RwLock<SearchTable>>,
    pub stop: Arc<AtomicBool>,
    pub pondering: Arc<AtomicBool>,
    pub number_threads: Arc<AtomicU16>,
}

pub fn uci_loop() {
    let mut engine = Engine {
        position: Position::from_fen(START_FEN).unwrap(),
        stop: Arc::new(AtomicBool::new(true)),
        game_history: Vec::new(),
        table: Arc::new(RwLock::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB))),
        pondering: Arc::new(AtomicBool::new(false)),
        number_threads: Arc::new(AtomicU16::new(DEFAULT_NUMBER_THREADS)),
    };

    println!("Camel {} by Bruno Mendes", env!("CARGO_PKG_VERSION"));

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
