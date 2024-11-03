use std::str::FromStr;

use position::{fen::START_POSITION, Position};

mod position;

fn main() {
    let position = Position::from_str(START_POSITION).unwrap();
    println!("{position}");
}
