use super::{Depth, MAX_DEPTH};
use crate::{
    evaluation::{Score, ValueScore},
    moves::Move,
    position::{board::ZobristHash, Position},
};
use std::{array, sync::RwLock};
use sync_unsafe_cell::SyncUnsafeCell;

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

type XoredZobristHash = ZobristHash;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScoreType {
    Exact = 0,
    LowerBound = 1, // when search fails high (beta cutoff)
    UpperBound = 2, // when search fails low (no improvement to alpha)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TableEntry {
    score: ValueScore,
    best_move: Move,
    score_type: ScoreType,
    depth: Depth,
    age: u8,
    pad: u8,
}

impl TableEntry {
    pub fn new(
        score: ValueScore,
        score_type: ScoreType,
        best_move: Move,
        depth: Depth,
        age: u8,
    ) -> Self {
        TableEntry { score, best_move, score_type, depth, age, pad: 0 }
    }

    pub fn shift_score(&self, shift: ValueScore) -> Self {
        TableEntry { score: self.score + shift, ..*self }
    }

    pub fn raw(&self) -> u64 {
        unsafe { std::mem::transmute(*self) }
    }
}

struct TranspositionTable {
    data: Vec<SyncUnsafeCell<Option<(TableEntry, XoredZobristHash)>>>,
    age: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self { data: (0..data_len).map(|_| SyncUnsafeCell::new(None)).collect(), age: 0 }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<(TableEntry, XoredZobristHash)>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| SyncUnsafeCell::new(None)).collect();
    }

    pub fn hashfull_millis(&self) -> usize {
        (0..10000).filter(|i| self.load_tt_entry(*i).is_some()).count() / 10
    }

    pub fn get(&self, position: &Position) -> Option<TableEntry> {
        let hash = position.zobrist_hash();
        let entry = self.load_tt_entry(hash as usize % self.data.len());
        entry.filter(|(entry, entry_hash)| entry_hash ^ entry.raw() == hash).map(|(entry, _)| entry)
    }

    pub fn insert(&self, position: &Position, entry: TableEntry, force: bool) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if !force {
            if let Some((old_entry, _)) = self.load_tt_entry(index) {
                if old_entry.depth > entry.depth && old_entry.age == entry.age {
                    return;
                }
            }
        }

        self.store_tt_entry(index, Some((entry, hash ^ entry.raw())));
    }

    fn load_tt_entry(&self, index: usize) -> Option<(TableEntry, XoredZobristHash)> {
        unsafe { *self.data[index].get() }
    }

    fn store_tt_entry(&self, index: usize, entry: Option<(TableEntry, XoredZobristHash)>) {
        unsafe { *self.data[index].get() = entry }
    }
}

pub struct SearchTable {
    transposition: RwLock<TranspositionTable>,
    killer_moves: [SyncUnsafeCell<Option<Move>>; 3 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new(size_mb: usize) -> Self {
        Self {
            transposition: RwLock::new(TranspositionTable::new(size_mb)),
            killer_moves: array::from_fn(|_| SyncUnsafeCell::new(None)),
        }
    }

    pub fn prepare_for_new_search(&self) {
        // We add to the age to be able to replace all entries from previous searches.
        // This is both faster and more effective than clearing the table completely,
        // since we can profit from older entries that are still valid.
        let mut tt = self.transposition.write().unwrap();
        tt.age = tt.age.wrapping_add(1);

        // Killer moves are no longer at the same ply, so we clear them.
        (0..self.killer_moves.len()).for_each(|index| self.store_killer(index, None));
    }

    pub fn set_size(&self, size_mb: usize) {
        self.transposition.write().unwrap().set_size(size_mb)
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition
            .read()
            .unwrap()
            .get(position)
            .map(|entry| entry.best_move)
            .filter(|mov| mov.is_pseudo_legal(position))
    }

    pub fn get_table_score(
        &self,
        position: &Position,
        depth: Depth,
        ply: Depth,
    ) -> Option<(ValueScore, ScoreType)> {
        self.transposition
            .read()
            .unwrap()
            .get(position)
            .and_then(|entry| {
                if entry.depth >= depth {
                    Some((entry.score, entry.score_type))
                } else {
                    None
                }
            })
            .map(|(score, score_type)| {
                // Adjust the score to the current distance from the root.
                if Score::is_mate(score) {
                    let shift = if score < 0 { ply as ValueScore } else { -(ply as ValueScore) };
                    (score + shift, score_type)
                } else {
                    (score, score_type)
                }
            })
    }

    pub fn insert_entry(
        &self,
        position: &Position,
        score: ValueScore,
        score_type: ScoreType,
        best_move: Move,
        depth: Depth,
        ply: Depth,
        is_root: bool,
    ) {
        let tt = self.transposition.read().unwrap();
        let entry = TableEntry::new(score, score_type, best_move, depth, tt.age);

        // The score stored should be independent of the path from root to this node,
        // and only depend on the number of moves to mate.
        if Score::is_mate(entry.score) {
            let shift = if entry.score > 0 { ply as ValueScore } else { -(ply as ValueScore) };
            let entry = entry.shift_score(shift);
            tt.insert(position, entry, is_root);
        } else {
            tt.insert(position, entry, is_root);
        }
    }

    pub fn put_killer_move(&self, ply: Depth, mov: Move) {
        let index = 2 * ply as usize;
        if self.load_killer(index).is_none() {
            self.store_killer(index, Some(mov));
        } else if self.load_killer(index + 1).is_none() {
            self.store_killer(index + 1, Some(mov));
        } else {
            self.store_killer(0, self.load_killer(index + 1));
            self.store_killer(index + 1, Some(mov));
        }
    }

    pub fn get_killers(&self, ply: Depth) -> [Option<Move>; 2] {
        let index = 2 * ply as usize;
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
            .for_each(|entry| *entry = SyncUnsafeCell::new(None));
        (0..self.killer_moves.len()).for_each(|index| self.store_killer(index, None));
    }

    fn load_killer(&self, index: usize) -> Option<Move> {
        unsafe { *self.killer_moves[index].get() }
    }

    fn store_killer(&self, index: usize, mov: Option<Move>) {
        unsafe { *self.killer_moves[index].get() = mov }
    }
}
