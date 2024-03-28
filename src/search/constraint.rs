use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use super::history::HistoryEntry;

pub struct TimeConstraint {
    pub initial_instant: Instant,
    pub move_time: Duration,
}

pub struct SearchConstraint {
    pub time_constraint: Option<TimeConstraint>,
    pub stop_now: Option<Arc<AtomicBool>>,
    pub ponder_mode: Option<Arc<AtomicBool>>,
    pub game_history: Vec<HistoryEntry>,
}

impl Default for SearchConstraint {
    fn default() -> Self {
        Self { time_constraint: None, stop_now: None, ponder_mode: None, game_history: vec![] }
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
}

#[cfg(test)]
mod tests {
    use super::SearchConstraint;
    use crate::search::constraint::TimeConstraint;
    use std::{
        thread,
        time::{Duration, Instant},
    };

    #[test]
    fn stop_search_time() {
        let constraint = SearchConstraint {
            time_constraint: Some(TimeConstraint {
                initial_instant: Instant::now(),
                move_time: Duration::from_millis(100),
            }),
            stop_now: None,
            ponder_mode: None,
            game_history: vec![],
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
            time_constraint: Some(TimeConstraint {
                initial_instant: Instant::now(),
                move_time: Duration::from_millis(100),
            }),
            stop_now: Some(stop_now.clone()),
            ponder_mode: None,
            game_history: vec![],
        };

        assert!(!constraint.should_stop_search());

        stop_now.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(constraint.should_stop_search());
        assert!(constraint.remaining_time().unwrap() > Duration::from_millis(90));
    }
}
