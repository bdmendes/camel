use super::MAX_DEPTH;
use crate::position::{board::ZobristHash, Position};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

#[derive(Debug, Copy, Clone)]
pub struct HistoryEntry {
    pub hash: ZobristHash,
    pub reversible: bool,
}

pub struct SearchConstraint {
    pub branch_history: Vec<HistoryEntry>,
    pub initial_instant: Option<Instant>,
    pub move_time: Option<Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl Default for SearchConstraint {
    fn default() -> Self {
        Self {
            branch_history: Vec::with_capacity(MAX_DEPTH as usize),
            initial_instant: None,
            move_time: None,
            stop_now: None,
        }
    }
}

impl SearchConstraint {
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

    pub fn visit_position(&mut self, position: &Position, reversible: bool) {
        self.branch_history.push(HistoryEntry { hash: position.zobrist_hash(), reversible });
    }

    pub fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    pub fn is_repetition<const TIMES: u8>(&self, position: &Position) -> bool {
        let mut count = 0;
        let hash = position.zobrist_hash();
        for entry in self.branch_history.iter().rev() {
            if entry.hash == hash {
                count += 1;
            }
            if count >= TIMES {
                return true;
            }
            if !entry.reversible {
                break;
            }
        }
        false
    }
}
