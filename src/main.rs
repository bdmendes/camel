use crate::core::Position;
use core::fen::START_POSITION;
use std::{str::FromStr, time::Instant};

#[allow(unused)]
mod core;

fn main() {
    let position = Position::from_str(START_POSITION).unwrap();

    for depth in 1..=10 {
        let time = Instant::now();
        let (nodes, _) = position.perft(depth);
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
