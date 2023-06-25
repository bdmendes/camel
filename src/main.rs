use camel::*;
use moves::attacks::magics::{BISHOP_MAGICS, ROOK_MAGICS};
use moves::gen::perft;
use position::{fen::KIWIPETE_WHITE_FEN, Position};

use once_cell::sync::Lazy;

fn main() {
    Lazy::force(&ROOK_MAGICS);
    Lazy::force(&BISHOP_MAGICS);

    let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
    for depth in 1..=5 {
        let time = std::time::Instant::now();
        let (nodes, _) = perft::<true>(&position, depth);
        let elapsed = time.elapsed().as_millis();
        println!(
            "Depth {}: {} in {} ms [{:.3} Mnps]",
            depth,
            nodes,
            elapsed,
            nodes as f64 / 1000.0 / (elapsed + 1) as f64
        );
    }
}
