use super::MAX_DEPTH;
use crate::position::board::ZobristHash;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

#[derive(Debug, Copy, Clone)]
pub struct HistoryEntry {
    pub hash: ZobristHash,
    pub reversible: bool,
}

pub struct TimeConstraint {
    pub initial_instant: Instant,
    pub move_time: Duration,
}

pub struct SearchConstraint {
    pub branch_history: Vec<HistoryEntry>,
    pub time_constraint: Option<TimeConstraint>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl Default for SearchConstraint {
    fn default() -> Self {
        Self {
            branch_history: Vec::with_capacity(MAX_DEPTH as usize),
            time_constraint: None,
            stop_now: None,
        }
    }
}

impl SearchConstraint {
    pub fn should_stop_search(&self) -> bool {
        if let Some(time_constraint) = &self.time_constraint {
            let elapsed = time_constraint.initial_instant.elapsed();
            elapsed >= time_constraint.move_time
        } else if let Some(stop_now) = &self.stop_now {
            stop_now.load(std::sync::atomic::Ordering::Relaxed)
        } else {
            false
        }
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.time_constraint.as_ref().map(|time_constraint| {
            time_constraint.move_time.saturating_sub(time_constraint.initial_instant.elapsed())
        })
    }

    pub fn visit_position(&mut self, hash: ZobristHash, reversible: bool) {
        self.branch_history.push(HistoryEntry { hash, reversible });
    }

    pub fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    pub fn repeated(&self, hash: ZobristHash) -> u8 {
        let mut count = 0;
        for entry in self.branch_history.iter().rev() {
            if entry.hash == hash {
                count += 1;
            }
            if !entry.reversible {
                break;
            }
        }
        count
    }
}
