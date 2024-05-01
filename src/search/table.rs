use super::{Depth, MAX_DEPTH};
use crate::{
    evaluation::{Score, ValueScore},
    moves::Move,
    position::Position,
};
use std::{
    array,
    mem::transmute,
    sync::{
        atomic::{AtomicU16, AtomicU64, Ordering},
        RwLock,
    },
};

pub const MAX_TABLE_SIZE_MB: usize = 2048;
pub const MIN_TABLE_SIZE_MB: usize = 1;
pub const DEFAULT_TABLE_SIZE_MB: usize = 64;

const NULL_KILLER: u16 = u16::MAX;
const NULL_TT_ENTRY: u64 = u64::MAX;

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
    data: u32, // bits 0-1: score type, bits 2-7: depth, bit 8: age, bits 9-31: hash
}

impl TableEntry {
    pub fn new(
        score: ValueScore,
        score_type: ScoreType,
        best_move: Move,
        depth: Depth,
        hash: u64,
        age: bool,
    ) -> Self {
        TableEntry {
            score,
            best_move,
            data: (score_type as u32)
                | ((depth as u32 & 0x3F) << 2)
                | ((age as u32) << 8)
                | ((hash as u32) << 9),
        }
    }

    pub fn from_raw(bytes: u64) -> Self {
        unsafe { transmute::<u64, TableEntry>(bytes) }
    }

    pub fn raw(&self) -> u64 {
        unsafe { transmute::<TableEntry, u64>(*self) }
    }

    pub fn shift_score(&self, shift: ValueScore) -> Self {
        TableEntry { score: self.score + shift, ..*self }
    }

    fn score_type(&self) -> ScoreType {
        match self.data & 3 {
            0 => ScoreType::Exact,
            1 => ScoreType::LowerBound,
            2 => ScoreType::UpperBound,
            _ => panic!("Invalid score type"),
        }
    }

    fn depth(&self) -> Depth {
        ((self.data >> 2) & 0x3F) as Depth
    }

    fn same_hash(&self, hash: u64) -> bool {
        // We store 23 bits of the hash, so we need to compare the first 23 bits.
        (self.data >> 9) == (hash as u32 & 0x7F_FFFF)
    }

    fn age(&self) -> bool {
        ((self.data >> 8) & 1) != 0
    }
}

struct TranspositionTable {
    data: Vec<AtomicU64>,
    age: bool,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let data_len = Self::calculate_data_len(size_mb);
        Self { data: (0..data_len).map(|_| AtomicU64::new(NULL_TT_ENTRY)).collect(), age: false }
    }

    fn calculate_data_len(size_mb: usize) -> usize {
        let element_size = std::mem::size_of::<Option<TableEntry>>();
        let size = size_mb * 1024 * 1024;
        size / element_size
    }

    pub fn set_size(&mut self, size_mb: usize) {
        let data_len = Self::calculate_data_len(size_mb);
        self.data = (0..data_len).map(|_| AtomicU64::new(NULL_TT_ENTRY)).collect();
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
        entry.filter(|entry| entry.same_hash(hash))
    }

    pub fn insert(&self, position: &Position, entry: TableEntry, force: bool) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if !force {
            if let Some(old_entry) = self.load_tt_entry(index) {
                if old_entry.depth() > entry.depth() && old_entry.age() == entry.age() {
                    return;
                }
            }
        }

        self.store_tt_entry(index, entry);
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

    pub fn prepare_for_new_search(&self) {
        let mut tt = self.transposition.write().unwrap();
        tt.age = !tt.age;
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
                if entry.depth() >= depth {
                    Some((entry.score, entry.score_type()))
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
        let entry = TableEntry::new(
            score,
            score_type,
            best_move,
            depth,
            position.zobrist_hash(),
            self.transposition.read().unwrap().age,
        );

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
            .for_each(|entry| *entry = AtomicU64::new(NULL_TT_ENTRY));
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
    use std::sync::atomic::Ordering;

    use super::{SearchTable, TableEntry, TranspositionTable};
    use crate::{
        moves::Move,
        position::{
            fen::{FromFen, START_FEN},
            square::Square,
            Position,
        },
        search::{
            table::{ScoreType, NULL_KILLER, NULL_TT_ENTRY},
            MAX_DEPTH,
        },
    };

    #[test]
    fn entry_packing() {
        let entry1 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), MAX_DEPTH, 0, true);
        let entry2 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), 0, 1, false);
        let entry3 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), 1, 2, true);
        let entry4 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), 2, 3, false);

        assert_eq!(entry1.depth(), MAX_DEPTH);
        assert_eq!(entry2.depth(), 0);
        assert_eq!(entry3.depth(), 1);
        assert_eq!(entry4.depth(), 2);

        assert_eq!(entry1.score, 100);
        assert_eq!(entry1.score_type(), ScoreType::Exact);

        assert!(entry1.age());
        assert!(!entry2.age());
        assert!(entry3.age());
        assert!(!entry4.age());
    }

    #[test]
    fn entry_transmutation() {
        let entry1 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), MAX_DEPTH, 0, true);
        let entry2 = TableEntry::new(100, ScoreType::Exact, Move::new_raw(0), 0, 1, false);

        assert!(entry1.raw() != entry2.raw());

        assert_eq!(TableEntry::from_raw(entry1.raw()), entry1);
        assert_eq!(TableEntry::from_raw(entry2.raw()), entry2);
    }

    #[test]
    fn tt_raw_contents() {
        let table = TranspositionTable::new(1);
        let position = Position::from_fen(START_FEN).unwrap();

        assert_eq!(table.data[0].load(Ordering::Relaxed), NULL_TT_ENTRY);
        assert_eq!(table.get(&position), None);

        let first_move = Move::new(Square::E2, Square::E4, crate::moves::MoveFlag::DoublePawnPush);
        let first_move_entry =
            TableEntry::new(100, ScoreType::Exact, first_move, 2, position.zobrist_hash(), true);

        table.insert(&position, first_move_entry, false);

        assert_eq!(
            table.data[position.zobrist_hash() as usize % table.data.len()].load(Ordering::Relaxed),
            first_move_entry.raw()
        );
        assert_eq!(table.get(&position).unwrap().best_move, first_move);
    }

    #[test]
    fn killers_raw_contents() {
        let table = SearchTable::new(1);

        assert_eq!(table.killer_moves[0].load(Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.killer_moves[1].load(Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.get_killers(0), [None, None]);

        let first_move = Move::new(Square::E2, Square::E4, crate::moves::MoveFlag::DoublePawnPush);
        let second_move = Move::new(Square::D2, Square::D4, crate::moves::MoveFlag::DoublePawnPush);
        let third_move = Move::new(Square::C2, Square::C4, crate::moves::MoveFlag::DoublePawnPush);

        table.put_killer_move(0, first_move);
        assert_eq!(table.killer_moves[0].load(Ordering::Relaxed), first_move.raw());
        assert_eq!(table.killer_moves[1].load(Ordering::Relaxed), NULL_KILLER);
        assert_eq!(table.get_killers(0), [Some(first_move), None]);

        table.put_killer_move(0, second_move);
        assert_eq!(table.killer_moves[0].load(Ordering::Relaxed), first_move.raw());
        assert_eq!(table.killer_moves[1].load(Ordering::Relaxed), second_move.raw());
        assert_eq!(table.get_killers(0), [Some(first_move), Some(second_move)]);

        table.put_killer_move(0, third_move);
        assert_eq!(table.killer_moves[0].load(Ordering::Relaxed), second_move.raw());
        assert_eq!(table.killer_moves[1].load(Ordering::Relaxed), third_move.raw());
        assert_eq!(table.get_killers(0), [Some(second_move), Some(third_move)]);
    }
}
