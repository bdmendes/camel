use std::collections::VecDeque;

use self::{
    executor::{
        execute_all_moves, execute_clear, execute_debug, execute_display, execute_do_move,
        execute_go, execute_help, execute_is_ready, execute_perft, execute_position, execute_quit,
        execute_set_option, execute_stop, execute_uci, execute_uci_new_game,
    },
    parser::{parse_debug, parse_domove, parse_go, parse_perft, parse_position, parse_set_option},
};

use super::{Command, Engine};

mod executor;
mod parser;

pub fn parse_command(input: &str) -> Option<Command> {
    let mut words = input.split_whitespace().collect::<VecDeque<_>>();
    let command = words.pop_front()?;

    match command {
        "position" => parse_position(&mut words),
        "go" => parse_go(&mut words),
        "stop" => Some(Command::Stop),
        "uci" => Some(Command::Uci),
        "debug" => parse_debug(&mut words),
        "isready" => Some(Command::IsReady),
        "ucinewgame" => Some(Command::UCINewGame),
        "setoption" => parse_set_option(&mut words),
        "perft" => parse_perft(&mut words),
        "domove" | "m" => parse_domove(&mut words),
        "display" | "d" => Some(Command::Display),
        "allmoves" | "l" => Some(Command::AllMoves),
        "help" | "h" => Some(Command::Help),
        "clear" | "c" => Some(Command::Clear),
        "quit" | "q" => Some(Command::Quit),
        _ => None,
    }
}

pub fn execute_command(command: Command, engine: &mut Engine) {
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
        Command::Uci => execute_uci(),
        Command::Debug(debug) => execute_debug(debug),
        Command::SetOption { name, value } => {
            execute_set_option(name.as_str(), value.as_str(), engine)
        }
        Command::IsReady => execute_is_ready(),
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
