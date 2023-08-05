use crate::position::Position;
use ahash::AHashMap;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

pub struct SearchConstraint {
    pub branch_history: AHashMap<Position, u8>,
    pub initial_instant: Option<Instant>,
    pub move_time: Option<Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl Default for SearchConstraint {
    fn default() -> Self {
        Self {
            branch_history: AHashMap::new(),
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

    pub fn visit_position(&mut self, position: &Position) {
        self.branch_history.entry(*position).and_modify(|entry| *entry += 1).or_insert(1);
    }

    pub fn leave_position(&mut self, position: &Position) {
        let entry = self.branch_history.get_mut(position).unwrap();

        if *entry == 1 {
            self.branch_history.remove(position);
        } else {
            *entry -= 1;
        }
    }

    pub fn is_threefold_repetition(&self, position: &Position) -> bool {
        if let Some(entry) = self.branch_history.get(position) {
            *entry >= 3
        } else {
            false
        }
    }

    pub fn is_twofold_repetition(&self, position: &Position) -> bool {
        if let Some(entry) = self.branch_history.get(position) {
            *entry >= 2
        } else {
            false
        }
    }
}
