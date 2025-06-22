use std::{process, str::FromStr, thread, time::Instant};

use clap_repl::ClapEditor;

use crate::{
    core::position::{
        Position,
        fen::{KIWIPETE_POSITION, START_POSITION},
    },
    engine::{
        Engine,
        repl::{
            Command, EmptyPrompt, PositionCommand, PositionStartposCommand, SetoptionNameCommand,
            SetoptionValueCommand,
        },
    },
    search::{SearchStatusValue, perft::perft, picker::MovePicker},
};

pub mod core;
pub mod engine;
pub mod evaluation;
pub mod search;

fn main() {
    let mut engine = Engine::default();
    println!("Camel {} by Bruno Mendes", env!("CARGO_PKG_VERSION"));

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
                let flattened = fen.join(" ");
                match Position::from_str(&flattened) {
                    Ok(position) => engine.position = position,
                    Err(_) => println!("Invalid FEN: {}", flattened),
                }
            }
            PositionCommand::Kiwi => {
                engine.position = Position::from_str(KIWIPETE_POSITION).unwrap()
            }
        },
        Command::Go { subcommands: _ } => {
            println!("Search is not yet implemented. Please use Camel 1.6.0 in the meantime!")
        }
        Command::Setoption { name } => match name {
            SetoptionNameCommand::Name { name, value } => match value {
                SetoptionValueCommand::Value { value } => match (name.as_str(), value.as_str()) {
                    ("UCI_Chess960", _) => (),
                    ("Ponder", _) => (),
                    _ => print!("Invalid option."),
                },
            },
        },
        Command::Evaluate => println!("{}cp", engine.evaluator.evaluate(&engine.position)),
        Command::Perft { depth } => {
            let status = engine.search_status.clone();
            status.set(SearchStatusValue::Searching);
            let _ = thread::spawn(move || {
                let now = Instant::now();
                let (nodes, _divided) = perft::<true>(&engine.position, depth, &status);
                println!("{} in {} millis", nodes, (Instant::now() - now).as_millis());
                status.set(SearchStatusValue::Stopped);
            });
        }
        Command::Stop => engine.search_status.set(SearchStatusValue::Stopped),
        Command::Ponderhit => match engine.search_status.get() {
            SearchStatusValue::Stopped | SearchStatusValue::Searching => {
                println!("I am not pondering.")
            }
            SearchStatusValue::Pondering => engine.search_status.set(SearchStatusValue::Searching),
        },
        Command::List => {
            let picker: MovePicker<'_> =
                MovePicker::new(&engine.position, false, None, [None, None]);
            for m in picker {
                print!("{} ", m);
            }
            println!();
        }
        Command::Display => print!("{}", engine.position),
        Command::Isready => println!("readyok"),
        Command::Uci => {
            println!("id name Camel {}", env!("CARGO_PKG_VERSION"));
            println!("id author Bruno Mendes");

            //println!(
            //    "option name Hash type spin default {} min {} max {}",
            //    DEFAULT_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB
            //);
            println!("option name UCI_Chess960 type check default true");
            println!("option name Ponder type check default true");

            println!("uciok");
        }
        Command::Exit => process::exit(0),
    });
}
