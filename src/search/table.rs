use super::{Depth, MAX_DEPTH};
use crate::{evaluation::ValueScore, moves::Move, position::Position};
use parking_lot::RwLock;
use std::array;

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableScore {
    Exact(ValueScore),
    LowerBound(ValueScore), // when search fails high (beta cutoff)
    UpperBound(ValueScore), // when search fails low (no improvement to alpha)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableEntry {
    pub depth: Depth,
    pub score: TableScore,
    pub best_move: Move,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TranspositionEntry {
    entry: TableEntry,
    hash: u64,
    full_move_number: u16,
    root: bool,
}

struct TranspositionTable {
    data: Vec<RwLock<Option<TranspositionEntry>>>,
    root_fullmove_number: u16,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self { data: (0..data_len).map(|_| RwLock::new(None)).collect(), root_fullmove_number: 0 }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| RwLock::new(None)).collect();
    }

    pub fn hashfull_millis(&self) -> usize {
        // The first 10000 elements should suffice as a good sample,
        // given that hashes should be different enough.
        self.data.iter().take(10000).filter(|entry| entry.read().is_some()).count() / 10
    }

    pub fn get(&self, position: &Position) -> Option<TranspositionEntry> {
        let hash = position.zobrist_hash();
        let entry = self.data[hash as usize % self.data.len()].read();
        entry.filter(|entry| entry.hash == hash)
    }

    pub fn insert(&self, position: &Position, entry: TableEntry, force: bool, root: bool) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if !force {
            if let Some(old_entry) = *self.data[index].read() {
                if (old_entry.entry.depth > entry.depth || root)
                    && old_entry.full_move_number >= self.root_fullmove_number
                {
                    return;
                }
            }
        }

        *self.data[index].write() = Some(TranspositionEntry {
            entry,
            hash,
            root,
            full_move_number: position.fullmove_number,
        });
    }
}

pub struct SearchTable {
    transposition: RwLock<TranspositionTable>,
    killer_moves: [RwLock<Option<Move>>; 2 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new(size_mb: usize) -> Self {
        Self {
            transposition: RwLock::new(TranspositionTable::new(size_mb)),
            killer_moves: array::from_fn(|_| RwLock::new(None)),
        }
    }

    pub fn prepare_for_new_search(&self, fullmove_number: u16) {
        self.transposition.write().root_fullmove_number = fullmove_number;
    }

    pub fn set_size(&self, size_mb: usize) {
        self.transposition.write().set_size(size_mb)
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition.read().get(position).map(|entry| entry.entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TableScore> {
        self.transposition.read().get(position).and_then(|entry| {
            if entry.entry.depth >= depth {
                Some(entry.entry.score)
            } else {
                None
            }
        })
    }

    pub fn insert_entry(&self, position: &Position, entry: TableEntry, force: bool, root: bool) {
        self.transposition.read().insert(position, entry, force, root);
    }

    pub fn put_killer_move(&self, depth: Depth, mov: Move) {
        let index = 2 * depth as usize;
        if self.killer_moves[index].read().is_none() {
            *self.killer_moves[index].write() = Some(mov);
        } else if self.killer_moves[index + 1].read().is_none() {
            *self.killer_moves[index + 1].write() = Some(mov);
        } else {
            *self.killer_moves[index].write() = *self.killer_moves[index + 1].read();
            *self.killer_moves[index + 1].write() = Some(mov);
        }
    }

    pub fn get_killers(&self, depth: Depth) -> [Option<Move>; 2] {
        let index = 2 * depth as usize;
        [*self.killer_moves[index].read(), *self.killer_moves[index + 1].read()]
    }

    pub fn get_pv(&self, position: &Position, mut depth: Depth) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut position = *position;

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
        self.transposition.read().hashfull_millis()
    }

    pub fn clear(&self) {
        self.transposition.read().data.iter().for_each(|entry| *entry.write() = None);
        self.killer_moves.iter().for_each(|entry| *entry.write() = None);
    }
}
