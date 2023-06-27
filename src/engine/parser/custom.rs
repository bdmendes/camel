use std::collections::VecDeque;

use crate::engine::Command;

pub fn parse_perft(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let depth = words.pop_front().ok_or(())?.parse::<u8>().map_err(|_| ())?;
    Ok(Command::Perft { depth })
}

pub fn parse_domove(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let mov_str = words.pop_front().ok_or(())?.to_string();
    Ok(Command::DoMove { mov_str })
}
