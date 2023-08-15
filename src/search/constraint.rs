use crate::position::Position;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HistoryEntry {
    pub position: Position,
    pub is_reversible: bool,
}

pub struct SearchConstraint {
    pub branch_history: Vec<HistoryEntry>,
    pub initial_instant: Option<Instant>,
    pub move_time: Option<Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
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

    pub fn visit_position(&mut self, position: &Position, is_reversible: bool) {
        self.branch_history.push(HistoryEntry { position: *position, is_reversible });
    }

    pub fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    pub fn is_repetition<const TIMES: u8>(&self, position: &Position) -> bool {
        let mut count = 0;
        for entry in self.branch_history.iter().rev() {
            if entry.position == *position {
                count += 1;
                if count >= TIMES {
                    return true;
                }
            }
            if !entry.is_reversible {
                break;
            }
        }
        false
    }
}
