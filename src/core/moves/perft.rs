use crate::core::{MoveStage, Position};

use super::{gen::generate_moves, make::make_move};

pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves(position, MoveStage::All);

    if depth == 1 {
        moves.len() as u64
    } else {
        moves
            .iter()
            .map(|mov| perft(&make_move::<true>(position, *mov), depth - 1))
            .sum()
    }
}
