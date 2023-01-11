use std::process::exit;

use crate::board::{Position, START_FEN};

pub enum UCICommand {
    UCI,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    Register(String),
    UCINewGame,
    Position(String, Vec<String>),
    Go(String),
    Stop,
    PonderHit,
    Quit,
}

pub struct EngineState {
    position: Position,
}

impl UCICommand {
    pub fn parse(input: &str) -> Result<UCICommand, String> {
        let mut tokens = input.split_whitespace();
        let command = tokens.next().ok_or("No command found")?;
        match command {
            "uci" => Ok(UCICommand::UCI),
            "debug" => {
                let value = tokens.next().ok_or("No value found")?;
                Ok(UCICommand::Debug(value == "on"))
            }
            "isready" => Ok(UCICommand::IsReady),
            "setoption" => {
                let name = tokens.next().ok_or("No name found")?;
                let value = tokens.next().ok_or("No value found")?;
                Ok(UCICommand::SetOption(name.to_string(), value.to_string()))
            }
            "register" => {
                let code = tokens.next().ok_or("No code found")?;
                Ok(UCICommand::Register(code.to_string()))
            }
            "ucinewgame" => Ok(UCICommand::UCINewGame),
            "position" => {
                let position = tokens.next().ok_or("No position found")?;
                let moves = tokens.map(|s| s.to_string()).collect();
                Ok(UCICommand::Position(position.to_string(), moves))
            }
            "go" => {
                let search = tokens.next().ok_or("No search found")?;
                Ok(UCICommand::Go(search.to_string()))
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
            position: Position::from_fen(START_FEN),
        }
    }

    pub fn execute(&self, command: UCICommand) {
        match command {
            UCICommand::UCI => Self::handle_uci(),
            UCICommand::Debug(value) => Self::handle_debug(value),
            UCICommand::IsReady => Self::handle_isready(),
            UCICommand::SetOption(name, value) => Self::handle_setoption(&name, &value),
            UCICommand::Register(code) => Self::handle_register(&code),
            UCICommand::UCINewGame => Self::handle_newgame(),
            UCICommand::Position(position, moves) => Self::handle_position(&position, moves),
            UCICommand::Go(search) => Self::handle_go(&search),
            UCICommand::Stop => Self::handle_stop(),
            UCICommand::PonderHit => Self::handle_ponderhit(),
            UCICommand::Quit => Self::handle_quit(),
        }
    }

    fn handle_uci() {
        println!("id name Camel");
        println!("id author Bruno Mendes");
        println!("uciok");
    }

    fn handle_isready() {
        println!("readyok");
    }

    fn handle_debug(value: bool) {
        todo!();
    }

    fn handle_setoption(name: &str, value: &str) {
        todo!();
    }

    fn handle_register(code: &str) {
        todo!();
    }

    fn handle_newgame() {
        todo!();
    }

    fn handle_position(position: &str, moves: Vec<String>) {
        todo!();
    }

    fn handle_go(search: &str) {
        todo!();
    }

    fn handle_stop() {
        todo!();
    }

    fn handle_ponderhit() {
        todo!();
    }

    fn handle_quit() {
        exit(0);
    }
}
