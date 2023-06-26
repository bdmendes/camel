use std::collections::HashMap;

use super::Depth;
use crate::{moves::Move, position::Position};

pub struct SearchTable {
    pub hash_move: HashMap<Position, (Move, Depth)>,
}

impl SearchTable {
    pub fn new() -> Self {
        Self { hash_move: HashMap::new() }
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.hash_move.get(position).map(|(mov, _)| *mov)
    }

    pub fn insert_hash_move(&mut self, position: Position, mov: Move, depth: Depth) {
        let current_depth = self.hash_move.get(&position).map(|(_, depth)| *depth);
        if current_depth.is_none() || current_depth.unwrap() < depth {
            self.hash_move.insert(position, (mov, depth));
        }
    }
}
