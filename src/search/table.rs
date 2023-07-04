use std::collections::HashMap;

use super::{Depth, MAX_DEPTH};
use crate::{
    evaluation::ValueScore,
    moves::{Move, MoveVec},
    position::Position,
};

const MAX_TABLE_SIZE: usize = 50_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TTScore {
    Exact(ValueScore),
    LowerBound(ValueScore), // when search fails high (beta cutoff)
    UpperBound(ValueScore), // when search fails low (no improvement to alpha)
}

pub struct TTEntry {
    pub depth: Depth,
    pub score: TTScore,
    pub best_move: Option<Move>,
}

pub struct SearchTable {
    transposition: HashMap<Position, TTEntry>,
    killer_moves: [Option<Move>; 2 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new() -> Self {
        Self { transposition: HashMap::new(), killer_moves: [None; 2 * (MAX_DEPTH + 1) as usize] }
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition.get(position).and_then(|entry| entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TTScore> {
        self.transposition.get(position).and_then(|entry| {
            if entry.depth >= depth {
                Some(entry.score)
            } else {
                None
            }
        })
    }

    pub fn insert_entry(&mut self, position: &Position, entry: TTEntry) {
        if let Some(old_entry) = self.transposition.get(position) {
            if old_entry.depth >= entry.depth {
                return;
            }
            if old_entry.depth == entry.depth && matches!(old_entry.score, TTScore::Exact(_)) {
                return;
            }
        }

        self.transposition.insert(position.clone(), entry);
    }

    pub fn put_killer_move(&mut self, depth: Depth, mov: Move) {
        let index = 2 * depth as usize;

        if self.killer_moves[index].is_none() {
            self.killer_moves[index] = Some(mov);
        } else if self.killer_moves[index + 1].is_none() {
            self.killer_moves[index + 1] = Some(mov);
        } else {
            self.killer_moves[index] = self.killer_moves[index + 1];
            self.killer_moves[index + 1] = Some(mov);
        }
    }

    pub fn get_killers(&self, depth: Depth) -> &[Option<Move>] {
        let index = 2 * depth as usize;
        &self.killer_moves[index..index + 2]
    }

    pub fn get_pv(&self, position: &Position, mut depth: Depth) -> MoveVec {
        let mut pv = MoveVec::new();
        let mut position = position.clone();

        while let Some(entry) = self.get_hash_move(&position) {
            pv.push(entry);
            depth -= 1;
            if depth == 0 {
                break;
            }
            position = position.make_move(entry);
        }

        pv
    }

    pub fn cleanup(&mut self) {
        if self.transposition.capacity() > MAX_TABLE_SIZE {
            self.transposition.clear();
        }
    }
}
