use super::Command;
use crate::position::{fen::START_FEN, Position};
use std::{collections::VecDeque, time::Duration};

pub fn parse_position(words: &mut VecDeque<&str>) -> Option<Command> {
    let mut fen = String::new();
    let mut position = Position::from_fen(START_FEN).unwrap();
    let mut game_history = Vec::new();

    while let Some(word) = words.pop_front() {
        match word {
            "fen" => {
                while let Some(word) = words.pop_front() {
                    if word == "moves" {
                        words.push_front(word);
                        break;
                    }
                    fen.push_str(word);
                    fen.push(' ');
                }

                if let Some(new_position) = Position::from_fen(&fen) {
                    position = new_position;
                } else {
                    return None;
                }
            }
            "moves" => {
                while let Some(mov_str) = words.pop_front() {
                    let actual_moves = position.moves::<false>();
                    if let Some(mov) = actual_moves.iter().find(|mov| mov.to_string() == mov_str) {
                        position = position.make_move(*mov);
                        game_history.push(position);
                    } else {
                        return None;
                    }
                }
            }
            "startpos" => (),
            _ => return None,
        }
    }

    Some(Command::Position { position, game_history })
}

pub fn parse_go(words: &mut VecDeque<&str>) -> Option<Command> {
    let mut depth = None;
    let mut move_time = None;
    let mut white_time = None;
    let mut black_time = None;
    let mut white_increment = None;
    let mut black_increment = None;

    loop {
        let word = words.pop_front();
        if word.is_none() {
            break;
        }
        let word = word.unwrap();
        match word {
            "depth" => {
                let value = words.pop_front()?;
                depth = Some(value.parse::<u8>().ok()?);
            }
            "movetime" => {
                let value = words.pop_front()?;
                move_time = Some(Duration::from_millis(value.parse::<u64>().ok()?));
            }
            "wtime" => {
                let value = words.pop_front()?;
                white_time = Some(Duration::from_millis(value.parse::<u64>().ok()?));
            }
            "btime" => {
                let value = words.pop_front()?;
                black_time = Some(Duration::from_millis(value.parse::<u64>().ok()?));
            }
            "winc" => {
                let value = words.pop_front()?;
                white_increment = Some(Duration::from_millis(value.parse::<u64>().ok()?));
            }
            "binc" => {
                let value = words.pop_front()?;
                black_increment = Some(Duration::from_millis(value.parse::<u64>().ok()?));
            }
            _ => {}
        }
    }

    Some(Command::Go { depth, move_time, white_time, black_time, white_increment, black_increment })
}

pub fn parse_perft(words: &mut VecDeque<&str>) -> Option<Command> {
    let depth = words.pop_front()?.parse::<u8>().ok()?;
    Some(Command::Perft { depth })
}

pub fn parse_domove(words: &mut VecDeque<&str>) -> Option<Command> {
    let mov_str = words.pop_front()?.to_string();
    Some(Command::DoMove { mov_str })
}

pub fn parse_debug(words: &mut VecDeque<&str>) -> Option<Command> {
    let word = words.pop_front()?;
    match word {
        "on" => Some(Command::Debug(true)),
        "off" => Some(Command::Debug(false)),
        _ => None,
    }
}

pub fn parse_set_option(words: &mut VecDeque<&str>) -> Option<Command> {
    if words.pop_front()? != "name" {
        return None;
    }

    let name = words.pop_front()?.to_string();

    if words.pop_front()? != "value" {
        return None;
    }

    let value = words.pop_front()?.to_string();

    Some(Command::SetOption { name, value })
}
