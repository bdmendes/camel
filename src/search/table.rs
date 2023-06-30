use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

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
    pub branch_history: Vec<Position>,
    pub initial_instant: Option<Instant>,
    pub move_time: Option<Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl SearchTable {
    pub fn new() -> Self {
        Self {
            transposition: HashMap::new(),
            killer_moves: HashMap::new(),
            branch_history: Vec::new(),
            initial_instant: None,
            move_time: None,
            stop_now: None,
        }
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

    pub fn should_stop_search(&self) -> bool {
        if let Some(move_time) = &self.move_time {
            let elapsed = self.initial_instant.unwrap().elapsed();
            elapsed >= *move_time
        } else if let Some(stop_now) = &self.stop_now {
            stop_now.load(std::sync::atomic::Ordering::Relaxed)
        } else {
            false
        }
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        if let Some(move_time) = self.move_time {
            let elapsed = self.initial_instant.unwrap().elapsed();
            if elapsed >= move_time {
                return Some(Duration::from_secs(0));
            }
            Some(move_time - elapsed)
        } else {
            None
        }
    }

    pub fn visit_position(&mut self, position: &Position) {
        self.branch_history.push(position.clone());
    }

    pub fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    pub fn is_threefold_repetition(&self, position: &Position) -> bool {
        let mut count = 0;
        for i in self.branch_history.iter().rev() {
            if i == position {
                count += 1;
                if count == 3 {
                    return true;
                }
            }
        }
        false
    }
}
