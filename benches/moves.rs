use std::str::FromStr;

use camel::{
    moves::gen::{generate_moves, MoveStage},
    position::{fen::KIWIPETE_POSITION, Position},
};

fn main() {
    divan::main();
}

#[divan::bench]
fn pawns() {
    let position = divan::black_box(Position::from_str(KIWIPETE_POSITION).unwrap());
    for _ in 0..=divan::black_box(100_000) {
        let _ = divan::black_box(generate_moves(&position, MoveStage::All));
    }
}
