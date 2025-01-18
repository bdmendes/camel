use crate::core::Position;
use core::fen::KIWIPETE_POSITION;
use std::{str::FromStr, time::Instant};

#[allow(dead_code)]
mod core;
mod search;

fn main() {
    let position = Position::from_str(KIWIPETE_POSITION).unwrap();

    for depth in 1..=10 {
        let time = Instant::now();
        let (nodes, div) = position.perft(depth);
        for (m, d) in div {
            println!("{}: {}", m, d);
        }
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
