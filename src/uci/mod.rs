mod time;

use std::{
    collections::VecDeque,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    position::Position,
    search::{search_iterative_deep, Depth},
};

use self::time::get_duration;

pub enum UCICommand {
    // Standard UCI commands
    UCI,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    Register(String),
    UCINewGame,
    PositionStart(VecDeque<String>),
    PositionFen(String, VecDeque<String>),
    Go {
        depth: Option<Depth>,
        move_time: Option<Duration>,
        white_time: Option<Duration>,
        black_time: Option<Duration>,
    },
    Stop,
    PonderHit,
    Quit,

    // Custom commands
    Display,
    Help,
    Move(String),
    AutoMove,
    List,
    Fen,
}

pub struct EngineState {
    debug: bool,
    position: Position,
    stop: Arc<AtomicBool>,
}
impl UCICommand {
    pub fn parse(input: &str) -> Result<UCICommand, String> {
        let mut tokens: VecDeque<String> =
            input.split_whitespace().map(|s| s.to_string()).collect();

        let command = tokens.pop_front().ok_or("No command found")?;

        match command.as_str() {
            "f" | "fen" => Ok(UCICommand::Fen),
            "l" | "list" => Ok(UCICommand::List),
            "a" | "automove" => Ok(UCICommand::AutoMove),
            "h" | "help" => Ok(UCICommand::Help),
            "m" | "move" => {
                let m = tokens.pop_front().ok_or("No move found")?;
                Ok(UCICommand::Move(m))
            }
            "d" | "display" => Ok(UCICommand::Display),
            "uci" => Ok(UCICommand::UCI),
            "debug" => {
                let value = tokens.pop_front().ok_or("No value found")?;
                Ok(UCICommand::Debug(value == "on"))
            }
            "isready" => Ok(UCICommand::IsReady),
            "setoption" => {
                let name = tokens.pop_front().ok_or("No name found")?;
                let value = tokens.pop_front().ok_or("No value found")?;
                Ok(UCICommand::SetOption(name.to_string(), value.to_string()))
            }
            "register" => {
                let code = tokens.pop_front().ok_or("No code found")?;
                Ok(UCICommand::Register(code.to_string()))
            }
            "ucinewgame" => Ok(UCICommand::UCINewGame),
            "position" => {
                let position = tokens.pop_front().ok_or("No position found")?;
                match position.as_str() {
                    "startpos" => Ok(UCICommand::PositionStart(tokens)),
                    "fen" => {
                        let mut fen = String::new();
                        while let Some(token) = tokens.pop_front() {
                            if token == "moves" {
                                break;
                            }
                            fen.push_str(token.as_str());
                            fen.push(' ');
                        }
                        Ok(UCICommand::PositionFen(fen, tokens))
                    }
                    _ => Err(format!(
                        "Unknown position command: {}; did you mean fen [...]?",
                        position
                    )),
                }
            }
            "go" => {
                let mut depth = None;
                let mut move_time = None;
                let mut white_time = None;
                let mut black_time = None;
                loop {
                    let token = tokens.pop_front();
                    if token.is_none() {
                        break;
                    }
                    let token = token.unwrap();
                    match token.as_str() {
                        "depth" => {
                            let value =
                                tokens.pop_front().ok_or("No value found")?;
                            depth = Some(
                                value
                                    .parse::<u8>()
                                    .map_err(|_| "Invalid depth value")?
                                    .try_into()
                                    .unwrap(),
                            );
                        }
                        "movetime" => {
                            let value =
                                tokens.pop_front().ok_or("No value found")?;
                            move_time = Some(Duration::from_millis(
                                value
                                    .parse::<u64>()
                                    .map_err(|_| "Invalid movetime value")?,
                            ));
                        }
                        "wtime" => {
                            let value =
                                tokens.pop_front().ok_or("No value found")?;
                            white_time = Some(Duration::from_millis(
                                value
                                    .parse::<u64>()
                                    .map_err(|_| "Invalid wtime value")?,
                            ));
                        }
                        "btime" => {
                            let value =
                                tokens.pop_front().ok_or("No value found")?;
                            black_time = Some(Duration::from_millis(
                                value
                                    .parse::<u64>()
                                    .map_err(|_| "Invalid btime value")?,
                            ));
                        }
                        _ => {}
                    }
                }
                Ok(UCICommand::Go { depth, move_time, white_time, black_time })
            }
            "stop" => Ok(UCICommand::Stop),
            "ponderhit" => Ok(UCICommand::PonderHit),
            "quit" => Ok(UCICommand::Quit),
            _ => Err(format!("Unknown command: {}", command)),
        }
    }
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            position: Position::new(),
            debug: false,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn execute(&mut self, command: UCICommand) {
        match command {
            UCICommand::Fen => self.handle_fen(),
            UCICommand::UCI => Self::handle_uci(),
            UCICommand::Debug(value) => self.handle_debug(value),
            UCICommand::IsReady => Self::handle_isready(),
            UCICommand::SetOption(name, value) => {
                Self::handle_setoption(&name, &value)
            }
            UCICommand::Register(code) => Self::handle_register(&code),
            UCICommand::UCINewGame => Self::handle_newgame(),
            UCICommand::PositionFen(fen, moves) => {
                self.handle_position(Some(&fen), moves)
            }
            UCICommand::PositionStart(moves) => {
                self.handle_position(None, moves)
            }
            UCICommand::Go { depth, move_time, white_time, black_time } => {
                self.handle_go(depth, move_time, white_time, black_time)
            }
            UCICommand::Stop => self.handle_stop(),
            UCICommand::PonderHit => Self::handle_ponderhit(),
            UCICommand::Quit => Self::handle_quit(),
            UCICommand::Display => self.handle_display(),
            UCICommand::Move(m) => self.handle_move(m),
            UCICommand::Help => Self::handle_help(),
            UCICommand::AutoMove => self.handle_auto_move(),
            UCICommand::List => self.handle_list(),
        }
    }

    fn handle_display(&self) {
        println!("{}", self.position);
    }

    fn handle_uci() {
        println!("id name Camel");
        println!("id author Bruno Mendes");
        println!("uciok");
    }

    fn handle_isready() {
        println!("readyok");
    }

    fn handle_debug(&mut self, _value: bool) {
        self.debug = _value;
    }

    fn handle_setoption(_name: &str, _value: &str) {}

    fn handle_register(_code: &str) {}

    fn handle_newgame() {}

    fn handle_position(
        &mut self,
        fen: Option<&str>,
        mut moves: VecDeque<String>,
    ) {
        if !moves.is_empty() && moves[0] == "moves" {
            moves.pop_front();
        }

        self.position = if let Some(fen) = fen {
            if let Ok(position) = Position::from_fen(fen) {
                position
            } else {
                println!("Invalid FEN: {}", fen);
                Position::new()
            }
        } else {
            Position::new()
        };

        for mov in moves {
            let legal_moves = self.position.legal_moves(false);
            if let Some(m) = legal_moves.iter().find(|m| m.to_string() == mov) {
                self.position = self.position.make_move(m);
            } else {
                println!("Invalid move: {}; stopping line before it", mov);
                break;
            }
        }
    }

    fn handle_go(
        &self,
        depth: Option<Depth>,
        move_time: Option<Duration>,
        mut white_time: Option<Duration>,
        mut black_time: Option<Duration>,
    ) {
        self.stop.store(false, Ordering::Relaxed);
        let stop_now = self.stop.clone();
        let position = self.position.clone();

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
            )),
            None => None,
        };

        thread::spawn(move || {
            search_iterative_deep(
                &position,
                depth,
                calc_move_time,
                Some(stop_now),
            );
        });
    }

    fn handle_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn handle_ponderhit() {}

    fn handle_quit() {
        exit(0);
    }

    fn handle_help() {
        println!("Camel is an UCI-compatible chess engine.");
        println!("You such use a GUI such as Arena (Windows) or Scid (Linux/Mac) to play against it.");
        println!("In alternative, check the UCI protocol details at https://backscattering.de/chess/uci/.");
        println!("Camel is written in Rust and is open source. You can find the source code at https://github.com/bdmendes/camel.");
        println!("");
        println!("Additional commands [besides standard UCI]:");
        println!("  help: display this help message");
        println!("  display: display the current position");
        println!("  move [long_algebraic_notation]: make a move in the current position");
        println!("  automove: let the engine make a move in the current position (1 second time limit)");
        println!("  list: list all legal moves in the current position");
        println!("  fen: display the FEN of the current position");
    }

    fn handle_move(&mut self, mov: String) {
        let legal_moves = self.position.legal_moves(false);
        if let Some(m) = legal_moves.iter().find(|m| m.to_string() == mov) {
            self.position = self.position.make_move(m);
            println!("{}", self.position);
        } else {
            println!("Invalid move: {}", mov);
        }
    }

    fn handle_auto_move(&mut self) {
        let (mov, _, _) = search_iterative_deep(
            &self.position,
            None,
            Some(Duration::from_secs(1)),
            None,
        );
        if let Some(m) = mov {
            self.position = self.position.make_move(&m);
            println!("{}", self.position);
        } else {
            println!("No move found. The game is over.");
        }
    }

    fn handle_list(&self) {
        let legal_moves = self.position.legal_moves(false);

        if legal_moves.is_empty() {
            println!("No legal moves. The game is over.");
            return;
        }

        for mov in legal_moves {
            print!("{} ", mov);
        }
        println!();
    }

    fn handle_fen(&self) {
        println!("{}", self.position.to_fen());
    }
}
