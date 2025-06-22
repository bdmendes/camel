use std::{process, str::FromStr, thread, time::Instant};

use crate::{
    core::position::{
        Position,
        fen::{KIWIPETE_POSITION, START_POSITION},
    },
    engine::{
        Engine,
        repl::{Command, PositionCommand, repl},
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

    repl(|cmd| {
        if engine.search_status.get() != SearchStatusValue::Stopped
            && matches!(
                cmd,
                Command::Go { .. } | Command::Perft { .. } | Command::Evaluate
            )
        {
            println!("I am searching. Stop the search first.");
            return;
        }

        match cmd {
            Command::Position { subcommand } => match subcommand {
                PositionCommand::Startpos { moves } => {
                    let position = moves
                        .iter()
                        .try_fold(Position::from_str(START_POSITION).unwrap(), |current, m| {
                            current.make_move_str(m)
                        });
                    match position {
                        Some(p) => engine.position = p,
                        None => println!("Invalid move sequence."),
                    }
                }
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
            Command::Go { .. } => {
                println!("Search is not yet implemented. Please use Camel 1.6.0 in the meantime!")
            }
            Command::Setoption { name, value } => match (name.as_str(), value) {
                ("UCI_Chess960", _) => (),
                ("Ponder", _) => (),
                _ => println!("Invalid option."),
            },
            Command::Evaluate => {
                println!(
                    "{}cp",
                    engine.evaluator.lock().unwrap().evaluate(&engine.position)
                )
            }
            Command::Perft { depth } => {
                let status = engine.search_status.clone();
                status.set(SearchStatusValue::Searching);
                let _ = thread::spawn(move || {
                    let now = Instant::now();
                    let (nodes, _divided) = perft::<true>(&engine.position, depth, &status);
                    status.set(SearchStatusValue::Stopped);
                    let elapsed = (Instant::now() - now).as_secs_f32();
                    println!(
                        "{} in {:.2}s ({:.0}Mnps)",
                        nodes,
                        elapsed,
                        (nodes as f32 / 1_000_000.0 / elapsed)
                    );
                });
            }
            Command::Stop => engine.search_status.set(SearchStatusValue::Stopped),
            Command::Ponderhit => match engine.search_status.get() {
                SearchStatusValue::Stopped | SearchStatusValue::Searching => {
                    println!("I am not pondering.")
                }
                SearchStatusValue::Pondering => {
                    engine.search_status.set(SearchStatusValue::Searching)
                }
            },
            Command::Ucinewgame => todo!("clear hash table."),
            Command::List => {
                let picker = MovePicker::new(&engine.position, false, None, [None, None]);
                let moves = picker
                    .into_iter()
                    .map(|m| format!("{} ", m))
                    .collect::<String>();
                println!("{}", moves);
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
            Command::Debug { .. } => (),
            Command::Quit => process::exit(0),
        }
    });
}
