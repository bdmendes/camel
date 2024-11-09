use std::{str::FromStr, time::Instant};

use camel::{
    moves::perft::perft,
    position::{fen::START_POSITION, Position},
};

fn main() {
    let position = Position::from_str(START_POSITION).unwrap();
    for depth in 1..=10 {
        let time = Instant::now();
        let nodes = perft(&position, depth);
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
