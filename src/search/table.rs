use super::{Depth, MAX_DEPTH};
use crate::{evaluation::ValueScore, moves::Move, position::Position};
use portable_atomic::AtomicU128;
use std::{
    array,
    mem::{size_of, transmute},
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 2;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

const NULL_KILLER: u16 = u16::MAX;
const NULL_TT_ENTRY: u128 = u128::MAX;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableScore {
    Exact(ValueScore),
    LowerBound(ValueScore), // when search fails high (beta cutoff)
    UpperBound(ValueScore), // when search fails low (no improvement to alpha)
}

enum TableReplacementScheme {
    AlwaysReplace,
    ReplaceIfDepthGreaterOrEqual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TableEntry {
    depth: u8,
    score: TableScore,
    best_move: Move,
    hash: u64,
    root: bool,
}

impl TableEntry {
    pub fn new(score: TableScore, best_move: Move, depth: Depth, hash: u64, root: bool) -> Self {
        TableEntry { score, best_move, hash, depth, root }
    }

    pub fn from_raw(bytes: u128) -> Self {
        unsafe { transmute::<u128, TableEntry>(bytes) }
    }

    pub fn raw(&self) -> u128 {
        debug_assert!(size_of::<TableEntry>() == 16);
        unsafe { transmute::<TableEntry, u128>(*self) }
    }
}

struct TranspositionTable {
    data: Vec<AtomicU128>,
    replacement_scheme: TableReplacementScheme,
}

impl TranspositionTable {
    pub fn new(size_mb: usize, replacement_scheme: TableReplacementScheme) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self {
            data: (0..data_len).map(|_| AtomicU128::new(NULL_TT_ENTRY)).collect(),
            replacement_scheme,
        }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TableEntry>>();
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
            / 10
    }

    pub fn get(&self, position: &Position) -> Option<TableEntry> {
        let hash = position.zobrist_hash();
        let entry = self.load_tt_entry(hash as usize % self.data.len());
        entry.filter(|entry| entry.hash == hash)
    }

    pub fn insert(&self, position: &Position, entry: TableEntry) -> bool {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if matches!(self.replacement_scheme, TableReplacementScheme::ReplaceIfDepthGreaterOrEqual) {
            if let Some(old_entry) = self.load_tt_entry(index) {
                if old_entry.depth > entry.depth || (old_entry.root && !entry.root) {
                    return false;
                }
            }
        }

        self.store_tt_entry(index, entry);
        true
    }

    fn load_tt_entry(&self, index: usize) -> Option<TableEntry> {
        let entry = self.data[index].load(Ordering::Relaxed);
        if entry == NULL_TT_ENTRY {
            None
        } else {
            Some(TableEntry::from_raw(entry))
        }
    }

    fn store_tt_entry(&self, index: usize, entry: TableEntry) {
        self.data[index].store(entry.raw(), Ordering::Relaxed)
    }
}

pub struct SearchTable {
    transposition_always: RwLock<TranspositionTable>,
    transposition_depth: RwLock<TranspositionTable>,
    killer_moves: [AtomicU16; 2 * (MAX_DEPTH + 1) as usize],
}

impl SearchTable {
    pub fn new(size_mb: usize) -> Self {
        Self {
            transposition_always: RwLock::new(TranspositionTable::new(
                size_mb / 2,
                TableReplacementScheme::AlwaysReplace,
            )),
            transposition_depth: RwLock::new(TranspositionTable::new(
                size_mb / 2,
                TableReplacementScheme::ReplaceIfDepthGreaterOrEqual,
            )),
            killer_moves: array::from_fn(|_| AtomicU16::new(NULL_KILLER)),
        }
    }

    pub fn set_size(&self, size_mb: usize) {
        self.transposition_always.write().unwrap().set_size(size_mb / 2);
        self.transposition_depth.write().unwrap().set_size(size_mb / 2);
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        let entry = self
            .transposition_always
            .read()
            .unwrap()
            .get(position)
            .or_else(|| self.transposition_depth.read().unwrap().get(position));
        entry.map(|entry| entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TableScore> {
        let score = self
            .transposition_always
            .read()
            .unwrap()
            .get(position)
            .or_else(|| self.transposition_depth.read().unwrap().get(position));
        score.and_then(|entry| if entry.depth >= depth { Some(entry.score) } else { None })
    }

    pub fn insert_entry(
        &self,
        position: &Position,
        score: TableScore,
        best_move: Move,
        depth: Depth,
        root: bool,
    ) {
        let entry = TableEntry::new(score, best_move, depth, position.zobrist_hash(), root);
        if !self.transposition_depth.read().unwrap().insert(position, entry) {
            self.transposition_always.read().unwrap().insert(position, entry);
        }
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
        let mut pv = Vec::with_capacity(depth as usize);
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
        self.transposition_always.read().unwrap().hashfull_millis() / 2
            + self.transposition_depth.read().unwrap().hashfull_millis() / 2
    }

    pub fn clear(&self) {
        self.transposition_always
            .write()
            .unwrap()
            .data
            .iter_mut()
            .for_each(|entry| *entry = AtomicU128::new(NULL_TT_ENTRY));
        self.transposition_depth
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

#[cfg(test)]
mod tests {
    use super::{SearchTable, TableEntry, TableScore, TranspositionTable};
    use crate::{
        moves::Move,
        position::{
            fen::{FromFen, START_FEN},
            square::Square,
            Position,
        },
        search::{
            table::{TableReplacementScheme, NULL_KILLER, NULL_TT_ENTRY},
            MAX_DEPTH,
        },
    };

    #[test]
    fn entry_transmutation() {
        let entry1 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), MAX_DEPTH, 0, false);
        let entry2 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), 0, 0, false);
        assert!(entry1.raw() != entry2.raw());

        assert_eq!(TableEntry::from_raw(entry1.raw()), entry1);
        assert_eq!(TableEntry::from_raw(entry2.raw()), entry2);
    }

    #[test]
    fn tt_raw_contents() {
        let table = TranspositionTable::new(1, TableReplacementScheme::AlwaysReplace);
        let position = Position::from_fen(START_FEN).unwrap();

        assert_eq!(table.data[0].load(portable_atomic::Ordering::Relaxed), NULL_TT_ENTRY);
        assert_eq!(table.get(&position), None);

        let first_move = Move::new(Square::E2, Square::E4, crate::moves::MoveFlag::DoublePawnPush);
        let first_move_entry =
            TableEntry::new(TableScore::Exact(0), first_move, 2, position.zobrist_hash(), false);

        table.insert(&position, first_move_entry);

        assert_eq!(
            table.data[position.zobrist_hash() as usize % table.data.len()]
                .load(portable_atomic::Ordering::Relaxed),
            first_move_entry.raw()
        );
        assert_eq!(table.get(&position).unwrap().best_move, first_move);
    }

    #[test]
    fn killers_raw_contents() {
        let table = SearchTable::new(1);

        assert_eq!(table.killer_moves[0].load(portable_atomic::Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.killer_moves[1].load(portable_atomic::Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.get_killers(0), [None, None]);

        let first_move = Move::new(Square::E2, Square::E4, crate::moves::MoveFlag::DoublePawnPush);
        let second_move = Move::new(Square::D2, Square::D4, crate::moves::MoveFlag::DoublePawnPush);
        let third_move = Move::new(Square::C2, Square::C4, crate::moves::MoveFlag::DoublePawnPush);

        table.put_killer_move(0, first_move);
        assert_eq!(
            table.killer_moves[0].load(portable_atomic::Ordering::Relaxed),
            first_move.raw()
        );
        assert_eq!(table.killer_moves[1].load(portable_atomic::Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.get_killers(0), [Some(first_move), None]);

        table.put_killer_move(0, second_move);
        assert_eq!(
            table.killer_moves[0].load(portable_atomic::Ordering::Relaxed),
            first_move.raw()
        );
        assert_eq!(
            table.killer_moves[1].load(portable_atomic::Ordering::Relaxed),
            second_move.raw()
        );
        assert_eq!(table.get_killers(0), [Some(first_move), Some(second_move)]);

        table.put_killer_move(0, third_move);
        assert_eq!(
            table.killer_moves[0].load(portable_atomic::Ordering::Relaxed),
            second_move.raw()
        );
        assert_eq!(
            table.killer_moves[1].load(portable_atomic::Ordering::Relaxed),
            third_move.raw()
        );
        assert_eq!(table.get_killers(0), [Some(second_move), Some(third_move)]);
    }
}
