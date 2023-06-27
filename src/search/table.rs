use std::collections::HashMap;

use super::Depth;
use crate::{
    evaluation::ValueScore,
    moves::{Move, MoveVec},
    position::Position,
};

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
    pub transposition: HashMap<Position, TTEntry>,
    pub killer_moves: HashMap<Depth, [Option<Move>; 2]>,
}

impl SearchTable {
    pub fn new() -> Self {
        Self { transposition: HashMap::new(), killer_moves: HashMap::new() }
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
        let entry = self.killer_moves.entry(depth).or_insert([None, None]);

        if entry[0].is_none() {
            entry[0] = Some(mov);
        } else if entry[1].is_none() {
            entry[1] = Some(mov);
        } else {
            entry[0] = entry[1];
            entry[1] = Some(mov);
        }
    }

    pub fn get_killers(&mut self, depth: Depth) -> [Option<Move>; 2] {
        self.killer_moves.entry(depth).or_insert([None, None]).clone()
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
}
