use std::{
    process,
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        Engine,
        repl::{Command, DumpCommand, PositionCommand, repl},
        time::get_duration,
    },
    moves::generate::magics::save_magics,
    position::{
        MoveStage, Position,
        color::Color,
        fen::{KIWIPETE_POSITION, START_POSITION},
        hash::save_zobrist_numbers,
    },
    search::{
        game_history::GameHistory,
        perft::perft,
        score_table::{DEFAULT_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB},
        status::SearchStatusValue,
    },
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
                    let Some((history, position)) = GameHistory::from_moves(
                        &Position::from_str(START_POSITION).unwrap(),
                        &moves.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                    ) else {
                        println!("Invalid move sequence.");
                        return;
                    };

                    engine.position = position;
                    *engine.game_history.lock().unwrap() = history;
                }
                PositionCommand::Fen { fen, moves } => {
                    let flattened = fen.join(" ");
                    let Ok(position) = Position::from_str(&flattened) else {
                        println!("Invalid FEN: {}", flattened);
                        return;
                    };

                    let Some((history, position)) =
                        GameHistory::from_moves(&position, &moves.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    else {
                        println!("Invalid move sequence.");
                        return;
                    };

                    engine.position = position;
                    *engine.game_history.lock().unwrap() = history;
                }
                PositionCommand::Kiwi => {
                    let position = Position::from_str(KIWIPETE_POSITION).unwrap();
                    engine.position = position;
                    *engine.game_history.lock().unwrap() = GameHistory::new(&position);
                }
            },
            Command::Go {
                depth,
                wtime,
                btime,
                winc,
                binc,
                ponder,
                movetime,
                ..
            } => {
                let status = engine.search_status.clone();
                status.set(if ponder { SearchStatusValue::Pondering } else { SearchStatusValue::Searching });

                let (time, increment) = match engine.position.side_to_move() {
                    Color::White => (wtime, winc),
                    Color::Black => (btime, binc),
                };

                let duration = movetime.map(Duration::from_millis).or_else(|| {
                    time.map(|t| {
                        get_duration(
                            &engine.position,
                            Duration::from_millis(t),
                            increment.map(Duration::from_millis),
                            ponder,
                        )
                    })
                });

                engine.go(depth, duration);
            }
            Command::Setoption { name, value } => match (name.as_str(), value) {
                ("Hash", number) => {
                    if let Some(size_mb) = number.and_then(|n| n.parse::<usize>().ok()) {
                        let mut table = engine.score_table.lock().unwrap();
                        table.clear();
                        table.resize(size_mb.clamp(MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB));
                    }
                }
                ("ClearHash", None) => engine.score_table.lock().unwrap().clear(),
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
                    let (nodes, _) = perft(&engine.position, depth, &status);
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
                let mut moves = engine.position.moves(MoveStage::All);
                let fmt = moves.into_iter().map(|m| format!("{} ", m)).collect::<String>();
                println!("{}\n{} moves", fmt, moves.len());
            }
            Command::Move { r#move } => {
                let mut moves = engine.position.moves(MoveStage::All);
                if let Some(mov) = moves.into_iter().find(|&m| m.to_string() == r#move) {
                    let reversible = mov.is_reversible(&engine.position);
                    engine.position = engine.position.make_move(mov);
                    engine.game_history.lock().unwrap().push(&engine.position, reversible);
                } else {
                    println!("{} is not a valid move in the current position.", r#move);
                }
            }
            Command::Display => print!("{}", engine.position),
            Command::Isready => println!("readyok"),
            Command::Uci => {
                println!("id name Camel {}", env!("CARGO_PKG_VERSION"));
                println!("id author Bruno Mendes");

                println!(
                    "option name Hash type spin default {} min {} max {}",
                    DEFAULT_TABLE_SIZE_MB, MIN_TABLE_SIZE_MB, MAX_TABLE_SIZE_MB
                );
                println!("option name ClearHash type button");
                println!("option name UCI_Chess960 type check default true");
                println!("option name Ponder type check default true");

                println!("uciok");
            }
            Command::Debug { .. } => (),
            Command::Dump { subcommand } => {
                let time_str = chrono::Local::now().format("%Y%m%d-%H%M%S");
                match subcommand {
                    DumpCommand::Zobrist => match save_zobrist_numbers(format!("{}.zobrist", time_str).as_str()) {
                        Ok(_) => println!("Saved Zobrist hashes in the current directory."),
                        Err(e) => println!("An error occurred: {}", e),
                    },
                    DumpCommand::Magics => match save_magics(
                        format!("{}.rmagics", time_str).as_str(),
                        format!("{}.bmagics", time_str).as_str(),
                    ) {
                        Ok(_) => println!("Saved rook and bishop magics in the current directory."),
                        Err(e) => println!("An error occurred: {}", e),
                    },
                };
            }
            Command::Quit => process::exit(0),
        }
    });
}
