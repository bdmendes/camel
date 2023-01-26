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

pub enum UCICommand {
    UCI,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    Register(String),
    UCINewGame,
    PositionStart(VecDeque<String>),
    PositionFen(String, VecDeque<String>),
    Go { depth: Option<Depth>, movetime: Option<Duration> },
    Stop,
    PonderHit,
    Quit,
    Display,
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
                let mut movetime = None;
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
                            movetime = Some(Duration::from_millis(
                                value
                                    .parse::<u64>()
                                    .map_err(|_| "Invalid movetime value")?,
                            ));
                        }
                        _ => {
                            return Err(format!(
                                "Unknown go command: {}",
                                token
                            ))
                        }
                    }
                }
                Ok(UCICommand::Go { depth, movetime })
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
            UCICommand::Display => self.handle_display(),
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
            UCICommand::Go { depth, movetime } => {
                self.handle_go(depth, movetime)
            }
            UCICommand::Stop => self.handle_stop(),
            UCICommand::PonderHit => Self::handle_ponderhit(),
            UCICommand::Quit => Self::handle_quit(),
        }
    }

    fn handle_display(&self) {
        println!("{}", self.position)
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

    fn handle_setoption(_name: &str, _value: &str) {
        todo!();
    }

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
            let legal_moves = self.position.legal_moves();
            if let Some(m) = legal_moves.iter().find(|m| m.to_string() == mov) {
                self.position = self.position.make_move(*m);
            } else {
                println!("Invalid move: {}; stopping line before it", mov);
                break;
            }
        }
    }

    fn handle_go(&self, depth: Option<Depth>, movetime: Option<Duration>) {
        self.stop.store(false, Ordering::Relaxed);
        let stop_now = self.stop.clone();
        let position = self.position.clone();
        thread::spawn(move || {
            search_iterative_deep(&position, depth, movetime, Some(stop_now));
        });
    }

    fn handle_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    fn handle_ponderhit() {
        todo!();
    }

    fn handle_quit() {
        exit(0);
    }
}
