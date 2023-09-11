use super::{Depth, MAX_DEPTH};
use crate::{evaluation::ValueScore, moves::Move, position::Position};

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
    full_move_number: u8,
}

struct TranspositionTable {
    data: Vec<Option<TranspositionEntry>>,
    size: usize,
    root_fullmove_number: u8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self {
            data: (0..data_len).map(|_| None).collect(),
            size: data_len,
            root_fullmove_number: 0,
        }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TranspositionEntry>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| None).collect();
        self.size = data_len;
    }

    pub fn hashfull_millis(&self) -> usize {
        self.data.iter().filter(|entry| entry.is_some()).count() * 1000 / self.size
    }

    pub fn get(&self, position: &Position) -> Option<TranspositionEntry> {
        let hash = position.zobrist_hash();
        let entry = self.data[hash as usize % self.size];
        entry.filter(|entry| entry.hash == hash)
    }

    pub fn insert<const FORCE: bool>(&mut self, position: &Position, entry: TableEntry) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.size;

        if !FORCE {
            if let Some(old_entry) = self.data[index] {
                if old_entry.entry.depth > entry.depth
                    && old_entry.full_move_number >= self.root_fullmove_number
                {
                    return;
                }
            }
        }

        self.data[index] = Some(TranspositionEntry {
            entry,
            hash,
            full_move_number: position.fullmove_number as u8,
        });
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

    pub fn prepare_for_new_search(&mut self, fullmove_number: u16) {
        self.transposition.root_fullmove_number = fullmove_number as u8;
    }

    pub fn set_size(&mut self, size_mb: usize) {
        self.transposition.set_size(size_mb)
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition.get(position).map(|entry| entry.entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TableScore> {
        self.transposition.get(position).and_then(|entry| {
            if entry.entry.depth >= depth {
                Some(entry.entry.score)
            } else {
                None
            }
        })
    }

    pub fn insert_entry<const FORCE: bool>(&mut self, position: &Position, entry: TableEntry) {
        self.transposition.insert::<FORCE>(position, entry);
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

    pub fn get_killers(&self, depth: Depth) -> [Option<Move>; 2] {
        let index = 2 * depth as usize;
        [self.killer_moves[index], self.killer_moves[index + 1]]
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
        self.transposition.hashfull_millis()
    }

    pub fn clear(&mut self) {
        self.transposition.data.iter_mut().for_each(|entry| *entry = None);
        self.killer_moves.iter_mut().for_each(|entry| *entry = None);
    }
}
