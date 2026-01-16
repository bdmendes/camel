use std::{process, str::FromStr, thread, time::Instant};

use crate::{
    engine::{
        Engine,
        repl::{Command, PositionCommand, repl},
    },
    moves::Move,
    position::{
        Position,
        fen::{KIWIPETE_POSITION, START_POSITION},
    },
    search::{game_history::GameHistory, perft::perft, picker::MovePicker, status::SearchStatusValue},
};

pub mod engine;
pub mod evaluation;
pub mod moves;
pub mod position;
pub mod search;

fn main() {
    let mut engine = Engine::default();
    println!("Camel {} by Bruno Mendes", env!("CARGO_PKG_VERSION"));

    repl(|cmd| {
        if engine.search_status.get() != SearchStatusValue::Stopped
            && matches!(cmd, Command::Go { .. } | Command::Perft { .. } | Command::Evaluate)
        {
            println!("I am searching. Stop the search first.");
            return;
        }

        match cmd {
            Command::Position { subcommand } => match subcommand {
                PositionCommand::Startpos { moves } => {
                    let mut position = Position::from_str(START_POSITION).unwrap();
                    let mut history = GameHistory::default();
                    history.push(&position, false);
                    let mut valid = true;

                    for mov in &moves {
                        if let Some(m) = position.get_move_str(mov) {
                            let reversible = m.is_reversible(&position);
                            position = position.make_move(m);
                            history.push(&position, reversible);
                        } else {
                            println!("Invalid move sequence.");
                            valid = false;
                            break;
                        }
                    }

                    if valid {
                        engine.position = position;
                        *engine.game_history.lock().unwrap() = history;
                    }
                }
                PositionCommand::Fen { fen } => {
                    let flattened = fen.join(" ");
                    match Position::from_str(&flattened) {
                        Ok(position) => engine.position = position,
                        Err(_) => println!("Invalid FEN: {}", flattened),
                    }
                }
                PositionCommand::Kiwi => engine.position = Position::from_str(KIWIPETE_POSITION).unwrap(),
            },
            Command::Go { .. } => {
                println!("Search is in alpha! Please use Camel 1.6.0 in the meantime!");
                engine.go();
            }
            Command::Setoption { name, value } => match (name.as_str(), value) {
                ("UCI_Chess960", _) => (),
                ("Ponder", _) => (),
                _ => println!("Invalid option."),
            },
            Command::Evaluate => {
                let mut evaluator = engine.evaluator.lock().unwrap();
                let time = Instant::now();
                let eval = evaluator.evaluate(&engine.position);
                println!("{}cp ({}μs)", eval, time.elapsed().as_micros())
            }
            Command::Perft { depth } => {
                let status = engine.search_status.clone();
                status.set(SearchStatusValue::Searching);
                let _ = thread::spawn(move || {
                    let time = Instant::now();
                    let (nodes, _divided) = perft::<true>(&engine.position, depth, &status);
                    status.set(SearchStatusValue::Stopped);
                    let elapsed = time.elapsed().as_secs_f32();
                    println!("{} in {:.2}s ({:.0}Mnps)", nodes, elapsed, (nodes as f32 / 1_000_000.0 / elapsed));
                });
            }
            Command::Stop => match engine.search_status.get() {
                SearchStatusValue::Stopped => println!("I am not searching."),
                _ => engine.search_status.set(SearchStatusValue::Stopped),
            },
            Command::Ponderhit => match engine.search_status.get() {
                SearchStatusValue::Stopped | SearchStatusValue::Searching => {
                    println!("I am not pondering.")
                }
                SearchStatusValue::Pondering => engine.search_status.set(SearchStatusValue::Searching),
            },
            Command::Ucinewgame => {
                engine.score_table.lock().unwrap().clear();
            }
            Command::List => {
                let picker = MovePicker::new(&engine.position, false, None, [None, None]);
                let moves = picker.into_iter().collect::<Vec<Move>>();
                let fmt = moves.iter().map(|m| format!("{} ", m)).collect::<String>();
                println!("{}\n{} moves", fmt, moves.len());
            }
            Command::Move { r#move } => {
                let picker = MovePicker::new(&engine.position, false, None, [None, None]);
                if let Some(mov) = picker.into_iter().find(|&m| m.to_string() == r#move) {
                    engine.position = engine.position.make_move(mov);
                } else {
                    println!("{} is not a valid move in the current position.", r#move);
                }
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
