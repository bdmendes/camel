use std::collections::VecDeque;

use self::{
    custom::{parse_domove, parse_perft},
    uci::{parse_go, parse_position},
};

use super::Command;

mod custom;
mod uci;

pub fn parse_command(input: &str) -> Result<Command, ()> {
    let mut words = input.split_whitespace().collect::<VecDeque<_>>();
    let command = words.pop_front();

    if command.is_none() {
        return Err(());
    }

    match command.unwrap() {
        "position" => parse_position(&mut words),
        "go" => parse_go(&mut words),
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
