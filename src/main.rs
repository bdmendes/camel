use std::str::FromStr;

use camel::position::{fen::START_POSITION, Position};

fn main() {
    let position = Position::from_str(START_POSITION).unwrap();
    println!("{position}");
}
