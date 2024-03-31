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
    hash: u64,
}

impl TableEntry {
    pub fn new(
        score: TableScore,
        best_move: Move,
        depth: Depth,
        root: bool,
        search_id: u8,
        hash: u64,
    ) -> Self {
        TableEntry {
            score,
            best_move,
            hash,
            data: (depth & 0x3F) | ((root as u8 & 1) << 7) | ((search_id & 1) << 6),
        }
    }

    pub fn from_raw(bytes: u128) -> Self {
        unsafe { transmute::<u128, TableEntry>(bytes) }
    }

    pub fn raw(&self) -> u128 {
        debug_assert!(size_of::<TableEntry>() == 16);
        unsafe { transmute::<TableEntry, u128>(*self) }
    }

    fn same_search_parity(&self, id: u8) -> bool {
        ((self.data & 0x40) >> 6) == (id & 1)
    }

    fn is_root(&self) -> bool {
        ((self.data & 0x80) >> 7) == 1
    }

    fn depth(&self) -> Depth {
        self.data & 0x3F
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

    pub fn insert(&self, position: &Position, entry: TableEntry, current_id: u8) {
        let hash = position.zobrist_hash();
        let index = hash as usize % self.data.len();

        if !entry.is_root() {
            if let Some(old_entry) = self.load_tt_entry(index) {
                let replace = (old_entry.depth() <= entry.depth() && !old_entry.is_root())
                    || (old_entry.is_root() && !old_entry.same_search_parity(current_id));
                if !replace {
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

    pub fn prepare_for_new_search(&self, position: &Position) {
        let mut table = self.transposition.write().unwrap();
        if let Some(last_position) = table.last_position {
            if position.zobrist_hash() != last_position.zobrist_hash() {
                table.current_id = table.current_id.wrapping_add(1);
                table.last_position = Some(*position);
            }
        }
    }

    pub fn set_size(&self, size_mb: usize) {
        self.transposition.write().unwrap().set_size(size_mb)
    }

    pub fn get_hash_move(&self, position: &Position) -> Option<Move> {
        self.transposition.read().unwrap().get(position).map(|entry| entry.best_move)
    }

    pub fn get_table_score(&self, position: &Position, depth: Depth) -> Option<TableScore> {
        self.transposition.read().unwrap().get(position).and_then(|entry| {
            if entry.depth() >= depth {
                Some(entry.score)
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
        let entry =
            TableEntry::new(score, best_move, depth, root, tt.current_id, position.zobrist_hash());
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
            table::{NULL_KILLER, NULL_TT_ENTRY},
            MAX_DEPTH,
        },
    };

    #[test]
    fn entry_packing() {
        let entry1 =
            TableEntry::new(TableScore::Exact(100), Move::new_raw(0), MAX_DEPTH, true, 1, 0);
        let entry2 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), 0, true, 1, 0);
        let entry3 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), 1, false, 2, 0);
        let entry4 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), 2, false, 3, 0);

        assert_eq!(entry1.depth(), MAX_DEPTH);
        assert_eq!(entry2.depth(), 0);
        assert_eq!(entry3.depth(), 1);
        assert_eq!(entry4.depth(), 2);

        assert!(entry1.is_root());
        assert!(entry2.is_root());
        assert!(!entry3.is_root());
        assert!(!entry4.is_root());

        assert!(entry1.same_search_parity(1));
        assert!(!entry1.same_search_parity(2));
        assert!(entry1.same_search_parity(3));
        assert!(entry3.same_search_parity(2));
        assert!(!entry3.same_search_parity(3));
    }

    #[test]
    fn entry_transmutation() {
        let entry1 =
            TableEntry::new(TableScore::Exact(100), Move::new_raw(0), MAX_DEPTH, true, 1, 0);
        let entry2 = TableEntry::new(TableScore::Exact(100), Move::new_raw(0), 0, true, 1, 0);

        assert!(entry1.raw() != entry2.raw());

        assert_eq!(TableEntry::from_raw(entry1.raw()), entry1);
        assert_eq!(TableEntry::from_raw(entry2.raw()), entry2);
    }

    #[test]
    fn tt_raw_contents() {
        let table = TranspositionTable::new(1);
        let position = Position::from_fen(START_FEN).unwrap();

        assert_eq!(table.data[0].load(portable_atomic::Ordering::Relaxed), NULL_TT_ENTRY);
        assert_eq!(table.get(&position), None);

        let first_move = Move::new(Square::E2, Square::E4, crate::moves::MoveFlag::DoublePawnPush);
        let first_move_entry =
            TableEntry::new(TableScore::Exact(0), first_move, 2, true, 1, position.zobrist_hash());

        table.insert(&position, first_move_entry, 1);

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
