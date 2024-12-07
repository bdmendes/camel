use crate::core::{fen::START_POSITION, Position};
use core::MoveStage;
use std::{str::FromStr, time::Instant};

#[allow(unused)]
mod core;

fn main() {
    let position = Position::from_str(START_POSITION).unwrap();
    let moves = position.moves(MoveStage::All);
    moves.iter().for_each(|m| println!("{}", m));

    for depth in 1..=10 {
        let time = Instant::now();
        let nodes = position.perft(depth);
        let elapsed = time.elapsed().as_secs_f32();
        println!(
            "perft {}: {} [{} s; {} Mnps]",
            depth,
            nodes,
            time.elapsed().as_secs_f32(),
            ((nodes / 1000000).max(1)) as f32 / elapsed
        );
    }
}
