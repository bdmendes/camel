use std::collections::VecDeque;

use camel::position::{fen::START_FEN, Position};

use super::Command;

pub fn parse_position(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    let mut fen = String::new();
    let mut position = Position::from_fen(START_FEN).unwrap();

    while let Some(word) = words.pop_front() {
        match word {
            "fen" => {
                while let Some(word) = words.pop_front() {
                    if word == "moves" {
                        words.push_front(word);
                        break;
                    }
                    fen.push_str(&word);
                    fen.push(' ');
                }

                if let Ok(new_position) = Position::from_fen(&fen) {
                    position = new_position;
                } else {
                    return Err(());
                }
            }
            "moves" => {
                while let Some(mov_str) = words.pop_front() {
                    let actual_moves = position.moves::<false>();
                    if let Some(mov) = actual_moves.iter().find(|mov| mov.to_string() == mov_str) {
                        position = position.make_move(*mov);
                    } else {
                        return Err(());
                    }
                }
            }
            "startpos" => (),
            _ => return Err(()),
        }
    }

    Ok(Command::Position { position })
}

pub fn parse_go(words: &mut VecDeque<&str>) -> Result<Command, ()> {
    while let Some(word) = words.pop_front() {
        match word {
            "depth" => {
                let depth = words.pop_front().ok_or(())?.parse::<u8>().map_err(|_| ())?;
                return Ok(Command::Go { depth });
            }
            _ => (),
        }
    }

    Err(())
}
