use std::collections::VecDeque;

use self::{
    executor::{
        execute_all_moves, execute_clear, execute_display, execute_do_move, execute_go,
        execute_help, execute_perft, execute_position, execute_quit, execute_stop,
        execute_uci_new_game,
    },
    parser::{parse_domove, parse_go, parse_perft, parse_position},
};

use super::{Command, Engine};

mod executor;
mod parser;

pub fn parse_command(input: &str) -> Result<Command, ()> {
    let mut words = input.split_whitespace().collect::<VecDeque<_>>();
    let command = words.pop_front();

    if command.is_none() {
        return Err(());
    }

    match command.unwrap() {
        "position" => parse_position(&mut words),
        "go" => parse_go(&mut words).map_or(Result::Err(()), |cmd| Result::Ok(cmd)),
        "stop" => Ok(Command::Stop),
        "uci" => Ok(Command::UCI),
        "debug" => Ok(Command::Debug(true)),
        "isready" => Ok(Command::IsReady),
        "ucinewgame" => Ok(Command::UCINewGame),
        "perft" => parse_perft(&mut words),
        "domove" | "m" => parse_domove(&mut words),
        "display" | "d" => Ok(Command::Display),
        "allmoves" | "l" => Ok(Command::AllMoves),
        "help" | "h" => Ok(Command::Help),
        "clear" | "c" => Ok(Command::Clear),
        "quit" | "q" => Ok(Command::Quit),
        _ => Err(()),
    }
}

pub fn execute_command(command: Command, engine: &'static mut Engine) {
    match command {
        Command::Position { position, game_history } => {
            execute_position(&position, &game_history, engine)
        }
        Command::Go {
            depth,
            move_time,
            white_time,
            black_time,
            white_increment,
            black_increment,
        } => execute_go(
            engine,
            depth,
            move_time,
            white_time,
            black_time,
            white_increment,
            black_increment,
        ),
        Command::Stop => execute_stop(engine),
        Command::UCI => println!("id name Camel\nid author Bruno Mendes\nuciok"),
        Command::Debug(_) => (),
        Command::IsReady => println!("readyok"),
        Command::UCINewGame => execute_uci_new_game(engine),
        Command::Perft { depth } => execute_perft(depth, &engine.position),
        Command::DoMove { mov_str } => execute_do_move(&mov_str, &mut engine.position),
        Command::Display => execute_display(&engine.position),
        Command::AllMoves => execute_all_moves(&engine.position),
        Command::Help => execute_help(),
        Command::Clear => execute_clear(),
        Command::Quit => execute_quit(),
    }
}
