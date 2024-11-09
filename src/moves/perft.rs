use crate::position::Position;

use super::{
    gen::{generate_moves, MoveStage},
    make::make_move,
};

pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    generate_moves(position, MoveStage::All, &mut moves);

    moves
        .iter()
        .map(|mov| perft(&make_move(position, *mov), depth - 1))
        .sum()
}
