use ahash::AHashMap;

use super::{Depth, MAX_DEPTH};
use crate::{
    evaluation::ValueScore,
    moves::{Move, MoveVec},
    position::Position,
};

pub const MAX_TABLE_SIZE_MB: usize = 4096;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TTScore {
    Exact(ValueScore),
    LowerBound(ValueScore), // when search fails high (beta cutoff)
    UpperBound(ValueScore), // when search fails low (no improvement to alpha)
}

pub struct TableEntry {
    pub depth: Depth,
    pub score: TTScore,
    pub best_move: Option<Move>,
}

struct TranspositionTable(AHashMap<Position, TableEntry>);

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self(AHashMap::with_capacity(data_len))
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.0.clear();
        self.0.reserve(data_len);
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<TableEntry>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn hashfull_millis(&self) -> usize {
        self.0.len() * 1000 / self.0.capacity()
    }

    pub fn get(&self, position: &Position) -> Option<&TableEntry> {
        self.0.get(position)
    }

    pub fn insert(&mut self, position: &Position, entry: TableEntry) -> bool {
        if self.0.len() >= self.0.capacity() {
            return false;
        }

        self.0.insert(*position, entry);
        true
    }

    pub fn cleanup(&mut self, hashfull_millis_goal: usize, position: &Position, depth: Depth) {
        let may_happen_again = |p: &Position| {
            p.halfmove_clock < depth as u8
                || position.halfmove_clock.abs_diff(p.halfmove_clock) < depth as u8
        };

        self.0.retain(|k, _| may_happen_again(k));

        let mut current_depth = 1;
        while self.hashfull_millis() > hashfull_millis_goal {
            self.0.retain(|_, v| v.depth > current_depth);
            current_depth += 1;
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub struct SearchTable {
    transposition: TranspositionTable,
    killer_moves: [Option<Move>; 2 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new(size_mb: usize) -> Self {
        Self {
            transposition: TranspositionTable::new(size_mb),
            killer_moves: [None; 2 * (MAX_DEPTH + 1) as usize],
        }
    }

    pub fn set_size(&mut self, size_mb: usize) {
        self.transposition.set_size(size_mb)
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

    pub fn insert_entry(&mut self, position: &Position, entry: TableEntry) {
        if let Some(old_entry) = self.transposition.get(position) {
            if old_entry.depth >= entry.depth {
                return;
            }
            if old_entry.depth == entry.depth && matches!(old_entry.score, TTScore::Exact(_)) {
                return;
            }
        }

        self.transposition.insert(position, entry);
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

    pub fn hashfull_millis(&self) -> usize {
        self.transposition.hashfull_millis()
    }

    pub fn clear(&mut self) {
        self.transposition.clear();
        self.killer_moves.iter_mut().for_each(|entry| *entry = None)
    }

    pub fn cleanup(&mut self, hashfull_millis_goal: usize, position: &Position, depth: Depth) {
        self.transposition.cleanup(hashfull_millis_goal, position, depth);
    }
}
