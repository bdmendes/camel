use std::collections::VecDeque;

use camel::position::{fen::START_FEN, Position};

pub enum UCICommand {
    // Standard commands
    Position { position: Position },
    Go { depth: u8 },
    // Custom commands
    Perft { depth: u8 },
    DoMove { mov_str: String },
    Display,
    AllMoves,
    Clear,
    Quit,
}

pub fn parse_uci_command(command: &str) -> Result<UCICommand, ()> {
    let mut words = command.split_whitespace().collect::<VecDeque<_>>();
    let command = words.pop_front();

    if command.is_none() {
        return Err(());
    }

    match command.unwrap() {
        // Standard commands
        "position" => parse_position(&mut words),
        "go" => parse_go(&mut words),
        // Custom commands
        "perft" => parse_perft(&mut words),
        "domove" | "m" => parse_domove(&mut words),
        "display" | "d" => Ok(UCICommand::Display),
        "allmoves" | "l" => Ok(UCICommand::AllMoves),
        "clear" | "c" => Ok(UCICommand::Clear),
        "quit" | "q" => Ok(UCICommand::Quit),
        _ => Err(()),
    }
}

fn parse_position(words: &mut VecDeque<&str>) -> Result<UCICommand, ()> {
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

    Ok(UCICommand::Position { position })
}

fn parse_go(words: &mut VecDeque<&str>) -> Result<UCICommand, ()> {
    while let Some(word) = words.pop_front() {
        match word {
            "depth" => {
                let depth = words.pop_front().ok_or(())?.parse::<u8>().map_err(|_| ())?;
                return Ok(UCICommand::Go { depth });
            }
            _ => (),
        }
    }

    Err(())
}

fn parse_perft(words: &mut VecDeque<&str>) -> Result<UCICommand, ()> {
    let depth = words.pop_front().ok_or(())?.parse::<u8>().map_err(|_| ())?;
    Ok(UCICommand::Perft { depth })
}

fn parse_domove(words: &mut VecDeque<&str>) -> Result<UCICommand, ()> {
    let mov_str = words.pop_front().ok_or(())?.to_string();
    Ok(UCICommand::DoMove { mov_str })
}
