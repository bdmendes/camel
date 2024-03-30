use portable_atomic::AtomicU128;

use super::{Depth, MAX_DEPTH};
use crate::{evaluation::ValueScore, moves::Move, position::Position};
use std::{
    array,
    mem::{size_of, transmute},
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

const NULL_KILLER: u16 = u16::MAX;
const NULL_TT_ENTRY: u128 = u128::MAX;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableScore {
    Exact(ValueScore),
    LowerBound(ValueScore), // when search fails high (beta cutoff)
    UpperBound(ValueScore), // when search fails low (no improvement to alpha)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TableEntry {
    data: u8,
    score: TableScore,
    best_move: Move,
}

impl TableEntry {
    pub fn new(
        score: TableScore,
        best_move: Move,
        depth: Depth,
        root: bool,
        search_id: u8,
    ) -> Self {
        TableEntry {
            score,
            best_move,
            data: (depth & 0x3F) | ((root as u8 & 1) << 7) | ((search_id & 1) << 6),
        }
    }

    fn same_search(&self, id: u8) -> bool {
        ((self.data & 0x40) >> 6) == (id & 1)
    }

    fn is_root(&self) -> bool {
        ((self.data & 0x80) >> 7) == 1
    }

    fn depth(&self) -> Depth {
        self.data & 0x3F
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TranspositionEntry {
    entry: TableEntry,
    hash: u64,
}

impl TranspositionEntry {
    pub fn from_raw(bytes: u128) -> Self {
        unsafe { transmute::<u128, TranspositionEntry>(bytes) }
    }

    pub fn raw(&self) -> u128 {
        debug_assert!(size_of::<TranspositionEntry>() == 16);
        unsafe { transmute::<TranspositionEntry, u128>(*self) }
    }
}

struct TranspositionTable {
    data: Vec<AtomicU128>,
    current_id: u8,
    last_position: Option<Position>,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self {
            data: (0..data_len).map(|_| AtomicU128::new(NULL_TT_ENTRY)).collect(),
            current_id: 0,
            last_position: None,
        }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| AtomicU128::new(NULL_TT_ENTRY)).collect();
    }

    pub fn hashfull_millis(&self) -> usize {
        // The hash keys are disperse, so a small sample should suffice for a relevant statistic.
        self.data
            .iter()
            .take(10000)
            .filter(|entry| entry.load(Ordering::Relaxed) != NULL_TT_ENTRY)
            .count()
            * 10
    }

    pub fn get(&self, position: &Position) -> Option<TranspositionEntry> {
        let hash = position.zobrist_hash();
        let entry = self.load_tt_entry(hash as usize % self.data.len());
        entry.filter(|entry| entry.hash == hash)
    }

    pub fn insert(&self, position: &Position, entry: TableEntry, current_id: u8) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if !entry.is_root() {
            if let Some(old_entry) = self.load_tt_entry(index) {
                let replace = (old_entry.entry.depth() <= entry.depth()
                    && !old_entry.entry.is_root())
                    || (old_entry.entry.is_root() && !old_entry.entry.same_search(current_id));
                if !replace {
                    return;
                }
            }
        }

        self.store_tt_entry(index, TranspositionEntry { entry, hash });
    }

    fn load_tt_entry(&self, index: usize) -> Option<TranspositionEntry> {
        let entry = self.data[index].load(Ordering::Relaxed);
        if entry == NULL_TT_ENTRY {
            None
        } else {
            Some(TranspositionEntry::from_raw(entry))
        }
    }

    fn store_tt_entry(&self, index: usize, entry: TranspositionEntry) {
        self.data[index].store(entry.raw(), Ordering::Relaxed)
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

    pub fn prepare_for_new_search(&self, position: &Position) {
        let mut table = self.transposition.write().unwrap();
        if let Some(last_position) = table.last_position {
            if position.zobrist_hash() != last_position.zobrist_hash() {
                table.current_id = table.current_id.saturating_add(1);
                table.last_position = Some(*position);
            }
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
            if entry.entry.depth() >= depth {
                Some(entry.entry.score)
            } else {
                None
            }
        })
    }

    pub fn insert_entry(
        &self,
        position: &Position,
        score: TableScore,
        best_move: Move,
        depth: Depth,
        root: bool,
    ) {
        let tt = self.transposition.read().unwrap();
        let entry = TableEntry::new(score, best_move, depth, root, tt.current_id);
        tt.insert(position, entry, tt.current_id);
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
            .for_each(|entry| *entry = AtomicU128::new(NULL_TT_ENTRY));
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
