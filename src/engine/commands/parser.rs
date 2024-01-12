use super::Command;
use camel::{
    moves::gen::MoveStage,
    position::{fen::START_FEN, Position},
};
use std::{collections::VecDeque, time::Duration};

pub fn parse_position(words: &mut VecDeque<&str>) -> Result<Command, ()> {
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
                    return Err(());
                }
            }
            "moves" => {
                while let Some(mov_str) = words.pop_front() {
                    let actual_moves = position.moves(MoveStage::All);
                    if let Some(mov) = actual_moves.iter().find(|mov| mov.to_string() == mov_str) {
                        position = position.make_move(*mov);
                        game_history.push(position);
                    } else {
                        return Err(());
                    }
                }
            }
            "startpos" => (),
            _ => return Err(()),
        }
    }

    Ok(Command::Position { position, game_history })
}

pub fn parse_go(words: &mut VecDeque<&str>) -> Result<Command, String> {
    let mut depth = None;
    let mut move_time = None;
    let mut white_time = None;
    let mut black_time = None;
    let mut white_increment = None;
    let mut black_increment = None;
    let mut ponder = false;

    loop {
        let word = words.pop_front();
        if word.is_none() {
            break;
        }
        let word = word.unwrap();
        match word {
            "ponder" => {
                ponder = true;
            }
            "depth" => {
                let value = words.pop_front().ok_or("No value found")?;
                depth = Some(value.parse::<u8>().map_err(|_| "Invalid depth value")?);
            }
            "movetime" => {
                let value = words.pop_front().ok_or("No value found")?;
                move_time = Some(Duration::from_millis(
                    value.parse::<u64>().map_err(|_| "Invalid movetime value")?,
                ));
            }
            "wtime" => {
                let value = words.pop_front().ok_or("No value found")?;
                white_time = Some(Duration::from_millis(
                    value.parse::<u64>().map_err(|_| "Invalid wtime value")?,
                ));
            }
            "btime" => {
                let value = words.pop_front().ok_or("No value found")?;
                black_time = Some(Duration::from_millis(
                    value.parse::<u64>().map_err(|_| "Invalid btime value")?,
                ));
            }
            "winc" => {
                let value = words.pop_front().ok_or("No value found")?;
                white_increment = Some(Duration::from_millis(
                    value.parse::<u64>().map_err(|_| "Invalid winc value")?,
                ));
            }
            "binc" => {
                let value = words.pop_front().ok_or("No value found")?;
                black_increment = Some(Duration::from_millis(
                    value.parse::<u64>().map_err(|_| "Invalid binc value")?,
                ));
            }
            _ => {}
        }
    }

    Ok(Command::Go {
        depth,
        move_time,
        white_time,
        black_time,
        white_increment,
        black_increment,
        ponder,
    })
}

pub fn parse_perft(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let depth = words.pop_front().ok_or(())?.parse::<u8>().map_err(|_| ())?;
    Ok(Command::Perft(depth))
}

pub fn parse_domove(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let mov_str = words.pop_front().ok_or(())?.to_string();
    Ok(Command::DoMove { mov_str })
}

pub fn parse_debug(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let word = words.pop_front().ok_or(())?;
    match word {
        "on" => Ok(Command::Debug(true)),
        "off" => Ok(Command::Debug(false)),
        _ => Err(()),
    }
}

pub fn parse_set_option(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    if words.pop_front().ok_or(())? != "name" {
        return Err(());
    }

    let name = words.pop_front().ok_or(())?.to_string();

    if words.pop_front().ok_or(())? != "value" {
        return Err(());
    }

    let value = words.pop_front().ok_or(())?.to_string();

    Ok(Command::SetOption { name, value })
}
