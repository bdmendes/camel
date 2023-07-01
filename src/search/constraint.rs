use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use crate::position::Position;

pub struct SearchConstraint {
    pub branch_history: Vec<Position>,
    pub initial_instant: Option<Instant>,
    pub move_time: Option<Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl SearchConstraint {
    pub fn new() -> Self {
        Self { branch_history: Vec::new(), initial_instant: None, move_time: None, stop_now: None }
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
