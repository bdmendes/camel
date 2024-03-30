use super::{Depth, MAX_DEPTH};
use crate::{evaluation::ValueScore, moves::Move, position::Position};
use std::{
    array,
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

const NULL_KILLER: u16 = u16::MAX;

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
    pub move_number: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TranspositionEntry {
    entry: TableEntry,
    hash: u64,
}

struct TranspositionTable {
    data: Vec<RwLock<Option<TranspositionEntry>>>,
    size: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self { data: (0..data_len).map(|_| RwLock::new(None)).collect(), size: data_len }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| RwLock::new(None)).collect();
        self.size = data_len;
    }

    pub fn hashfull_millis(&self) -> usize {
        self.data.iter().filter(|entry| entry.read().unwrap().is_some()).count() * 1000 / self.size
    }

    pub fn get(&self, position: &Position) -> Option<TranspositionEntry> {
        let hash = position.zobrist_hash();
        let entry = self.data[hash as usize % self.size].read().unwrap();
        entry.filter(|entry| entry.hash == hash)
    }

    pub fn insert(&self, position: &Position, entry: TableEntry) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.size;

        if let Some(old_entry) = *self.data[index].read().unwrap() {
            if old_entry.entry.depth > entry.depth
                && old_entry.entry.move_number >= entry.move_number
            {
                return;
            }
        }

        *self.data[index].write().unwrap() = Some(TranspositionEntry { entry, hash });
    }
}

pub struct SearchTable {
    transposition: RwLock<TranspositionTable>,
    killer_moves: [AtomicU16; 2 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new(size_mb: usize) -> Self {
        Self {
            transposition: RwLock::new(TranspositionTable::new(size_mb)),
            killer_moves: array::from_fn(|_| AtomicU16::new(NULL_KILLER)),
        }
    }

    pub fn set_size(&self, size_mb: usize) {
        self.transposition.write().unwrap().set_size(size_mb)
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition.read().unwrap().get(position).map(|entry| entry.entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TableScore> {
        self.transposition.read().unwrap().get(position).and_then(|entry| {
            if entry.entry.depth >= depth {
                Some(entry.entry.score)
            } else {
                None
            }
        })
    }

    pub fn insert_entry(&self, position: &Position, entry: TableEntry) {
        self.transposition.read().unwrap().insert(position, entry);
    }

    pub fn put_killer_move(&self, depth: Depth, mov: Move) {
        let index = 2 * depth as usize;
        if self.load_killer(index).is_none() {
            self.store_killer(index, mov);
        } else if self.load_killer(index + 1).is_none() {
            self.store_killer(index + 1, mov);
        } else {
            self.store_killer(
                index,
                self.load_killer(index + 1).unwrap_or(Move::new_raw(NULL_KILLER)),
            );
            self.store_killer(index + 1, mov);
        }
    }

    pub fn get_killers(&self, depth: Depth) -> [Option<Move>; 2] {
        let index = 2 * depth as usize;
        [self.load_killer(index), self.load_killer(index + 1)]
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
        self.transposition.read().unwrap().hashfull_millis()
    }

    pub fn clear(&self) {
        self.transposition
            .write()
            .unwrap()
            .data
            .iter_mut()
            .for_each(|entry| *entry = RwLock::new(None));
        self.killer_moves.iter().for_each(|entry| entry.store(NULL_KILLER, Ordering::Relaxed));
    }

    fn load_killer(&self, index: usize) -> Option<Move> {
        let killer = self.killer_moves[index].load(Ordering::Relaxed);
        if killer == NULL_KILLER {
            None
        } else {
            Some(Move::new_raw(killer))
        }
    }

    fn store_killer(&self, index: usize, mov: Move) {
        self.killer_moves[index].store(mov.raw(), Ordering::Relaxed);
    }
}
