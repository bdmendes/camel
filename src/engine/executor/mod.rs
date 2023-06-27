use self::custom::{
    execute_all_moves, execute_clear, execute_display, execute_do_move, execute_perft, execute_quit,
};

use super::{Command, Engine};

pub mod custom;
pub mod uci;

pub fn execute_command(command: &Command, engine: &mut Engine) {
    match command {
        Command::Position { position } => todo!(),
        Command::Go { depth } => todo!(),
        Command::Perft { depth } => execute_perft(*depth, &engine.position),
        Command::DoMove { mov_str } => execute_do_move(mov_str, &mut engine.position),
        Command::Display => execute_display(&engine.position),
        Command::AllMoves => execute_all_moves(&engine.position),
        Command::Clear => execute_clear(),
        Command::Quit => execute_quit(),
    }
}
