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

pub struct TimeConstraint {
    pub initial_instant: Instant,
    pub move_time: Duration,
}

pub struct SearchConstraint {
    pub branch_history: Vec<HistoryEntry>,
    pub time_constraint: Option<TimeConstraint>,
    pub stop_now: Option<Arc<AtomicBool>>,
    pub ponder_mode: Option<Arc<AtomicBool>>,
}

impl Default for SearchConstraint {
    fn default() -> Self {
        Self {
            branch_history: Vec::with_capacity(MAX_DEPTH as usize),
            time_constraint: None,
            stop_now: None,
            ponder_mode: None,
        }
    }
}

impl SearchConstraint {
    pub fn should_stop_search(&self) -> bool {
        if let Some(ponder_mode) = &self.ponder_mode {
            if ponder_mode.load(std::sync::atomic::Ordering::Relaxed) {
                return false;
            }
        }

        if let Some(stop_now) = &self.stop_now {
            if stop_now.load(std::sync::atomic::Ordering::Relaxed) {
                return true;
            }
        }

        if let Some(time_constraint) = &self.time_constraint {
            let elapsed = time_constraint.initial_instant.elapsed();
            return elapsed >= time_constraint.move_time;
        }

        false
    }

    pub fn pondering(&self) -> bool {
        if let Some(ponder_mode) = &self.ponder_mode {
            return ponder_mode.load(std::sync::atomic::Ordering::Relaxed);
        }
        false
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.time_constraint.as_ref().map(|time_constraint| {
            time_constraint.move_time.saturating_sub(time_constraint.initial_instant.elapsed())
        })
    }

    pub fn visit_position(&mut self, position: &Position, reversible: bool) {
        self.branch_history.push(HistoryEntry { hash: position.zobrist_hash(), reversible });
    }

    pub fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    pub fn repeated(&self, position: &Position) -> u8 {
        let mut count = 0;
        let hash = position.zobrist_hash();
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

#[cfg(test)]
mod tests {
    use super::SearchConstraint;
    use crate::{
        position::{
            fen::{FromFen, START_FEN},
            Position,
        },
        search::constraint::TimeConstraint,
    };
    use std::{
        thread,
        time::{Duration, Instant},
    };

    #[test]
    fn stop_search_time() {
        let constraint = SearchConstraint {
            branch_history: Vec::new(),
            time_constraint: Some(TimeConstraint {
                initial_instant: Instant::now(),
                move_time: Duration::from_millis(100),
            }),
            stop_now: None,
            ponder_mode: None,
        };

        thread::sleep(Duration::from_millis(90));

        assert!(!constraint.should_stop_search());
        assert!(constraint.remaining_time().unwrap() < Duration::from_millis(10));

        thread::sleep(Duration::from_millis(20));

        assert!(constraint.should_stop_search());
        assert!(constraint.remaining_time().unwrap() == Duration::ZERO);
    }

    #[test]
    fn stop_search_external_order() {
        let stop_now = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let constraint = SearchConstraint {
            branch_history: Vec::new(),
            time_constraint: Some(TimeConstraint {
                initial_instant: Instant::now(),
                move_time: Duration::from_millis(100),
            }),
            stop_now: Some(stop_now.clone()),
            ponder_mode: None,
        };

        assert!(!constraint.should_stop_search());

        stop_now.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(constraint.should_stop_search());
        assert!(constraint.remaining_time().unwrap() > Duration::from_millis(90));
    }

    #[test]
    fn repeated_times() {
        let mut constraint = SearchConstraint {
            branch_history: Vec::new(),
            time_constraint: None,
            stop_now: None,
            ponder_mode: None,
        };

        let mut position = Position::from_fen(START_FEN).unwrap();
        constraint.visit_position(&position, true);

        position = position.make_move_str("e2e4").unwrap();
        constraint.visit_position(&position, false);

        position = position.make_move_str("e7e5").unwrap();
        constraint.visit_position(&position, false);

        assert_eq!(constraint.repeated(&position), 1);

        position = position.make_move_str("g1f3").unwrap();
        constraint.visit_position(&position, true);

        position = position.make_move_str("b8c6").unwrap();
        constraint.visit_position(&position, true);

        position = position.make_move_str("f3g1").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 1);

        position = position.make_move_str("c6b8").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 2);

        position = position.make_move_str("g1f3").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 2);

        position = position.make_move_str("b8c6").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 2);

        position = position.make_move_str("f3g1").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 2);

        position = position.make_move_str("c6b8").unwrap();
        constraint.visit_position(&position, true);

        assert_eq!(constraint.repeated(&position), 3);
    }
}
