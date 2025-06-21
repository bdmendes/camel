use std::{process, str::FromStr};

use clap::{Parser, Subcommand};
use clap_repl::{
    ClapEditor,
    reedline::{self},
};

use crate::{
    core::position::{
        MoveStage, Position,
        fen::{KIWIPETE_POSITION, START_POSITION},
    },
    engine::Engine,
};

pub mod core;
pub mod engine;
pub mod evaluation;
#[allow(dead_code)]
pub mod search;

struct EmptyPrompt;

impl reedline::Prompt for EmptyPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_indicator(
        &self,
        _prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        "".into()
    }
}

#[derive(Parser)]
#[command(name = "")]
enum Command {
    /// Set the current engine position.
    Position {
        #[command(subcommand)]
        subcommand: PositionCommand,
    },
    /// Search from the current position.
    Go { subcommands: Vec<String> },
    /// Statically evaluate the current position.
    Evaluate,
    /// List the moves available in the current position.
    List,
    /// Display the current position.
    Display,
    /// Respond when available.
    Isready,
    /// Identify the engine.
    Uci,
    /// Exit the process.
    Exit,
}

#[derive(Subcommand)]
enum PositionCommand {
    /// From a Forsyth–Edwards Notation string.
    Fen {
        /// The Forsyth–Edwards Notation describing the position.
        fen: Vec<String>,
    },
    /// From the starting position.
    Startpos {
        #[command(subcommand)]
        continuation: Option<PositionStartposCommand>,
    },
    /// The Kiwipete position.
    Kiwi,
}

#[derive(Subcommand)]
enum PositionStartposCommand {
    /// A sequence of moves from the start position in long algebraic notation. For example, "e4e5 g8f6".
    Moves {
        /// The sequence of moves after the starting position.
        moves: Vec<String>,
    },
}

fn main() {
    let mut engine = Engine::default();

    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(EmptyPrompt))
        .build();
    rl.repl(|cmd| match cmd {
        Command::Position { subcommand } => match subcommand {
            PositionCommand::Startpos { continuation } => match continuation {
                Some(PositionStartposCommand::Moves { moves }) => {
                    let position = moves
                        .iter()
                        .try_fold(engine.position, |current, m| current.make_move_str(m));
                    match position {
                        Some(p) => engine.position = p,
                        None => println!("Invalid move sequence."),
                    }
                }
                None => engine.position = Position::from_str(START_POSITION).unwrap(),
            },
            PositionCommand::Fen { fen } => {
                let joined_fen = fen.join(" ");
                match Position::from_str(&joined_fen) {
                    Ok(position) => engine.position = position,
                    Err(_) => println!("Invalid FEN: {}", joined_fen),
                }
            }
            PositionCommand::Kiwi => {
                engine.position = Position::from_str(KIWIPETE_POSITION).unwrap()
            }
        },
        Command::Go { subcommands: _ } => {
            println!("Search is not yet implemented. Please use Camel 1.6.0 in the meantime!")
        }
        Command::Evaluate => println!("{}cp", engine.evaluator.evaluate(&engine.position)),
        Command::List => {
            let moves = engine.position.moves(MoveStage::All);
            println!(
                "{}",
                moves
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        Command::Display => print!("{}", engine.position),
        Command::Isready => println!("readyok"),
        Command::Uci => {
            println!("id name Camel {}", env!("CARGO_PKG_VERSION"));
            println!("id author Bruno Mendes");

            //println!(
            //    "option name Threads type spin default {} min 1 max {}",
            //    DEFAULT_NUMBER_THREADS, MAX_THREADS
            //);
            //println!(
            //    "option name Hash type spin default {} min {} max {}",
            //    DEFAULT_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB
            //);
            println!("option name UCI_Chess960 type check default true",);
            //println!("option name Ponder type check default true",);

            println!("uciok");
        }
        Command::Exit => process::exit(0),
    });
}
