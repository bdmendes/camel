use super::history::HistoryEntry;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Copy, Clone)]
pub struct TimeConstraint {
    pub initial_instant: Instant,
    pub move_time: Duration,
}

#[derive(Default)]
pub struct SearchConstraint {
    pub time_constraint: Option<TimeConstraint>,
    pub global_stop: Arc<AtomicBool>,
    pub threads_stop: Arc<AtomicBool>,
    pub ponder_mode: Arc<AtomicBool>,
    pub number_threads: Arc<AtomicU16>,
    pub game_history: Vec<HistoryEntry>,
}

impl SearchConstraint {
    pub fn should_stop_search(&self) -> bool {
        if self.threads_stop.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }

        if self.ponder_mode.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }

        if self.global_stop.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }

        if let Some(time_constraint) = &self.time_constraint {
            let elapsed = time_constraint.initial_instant.elapsed();
            return elapsed >= time_constraint.move_time;
        }

        false
    }

    pub fn pondering(&self) -> bool {
        self.ponder_mode.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.time_constraint.as_ref().map(|time_constraint| {
            time_constraint.move_time.saturating_sub(time_constraint.initial_instant.elapsed())
        })
    }

    pub fn signal_root_finished(&self) {
        self.threads_stop.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::SearchConstraint;
    use crate::search::constraint::TimeConstraint;
    use std::{
        sync::{
            atomic::{AtomicBool, AtomicU16},
            Arc,
        },
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
            global_stop: Arc::new(AtomicBool::new(false)),
            threads_stop: Arc::new(AtomicBool::new(false)),
            ponder_mode: Arc::new(AtomicBool::new(false)),
            number_threads: Arc::new(AtomicU16::new(1)),
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
            global_stop: stop_now.clone(),
            threads_stop: Arc::new(AtomicBool::new(false)),
            ponder_mode: Arc::new(AtomicBool::new(false)),
            number_threads: Arc::new(AtomicU16::new(1)),
            game_history: vec![],
        };

        assert!(!constraint.should_stop_search());

        stop_now.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(constraint.should_stop_search());
        assert!(constraint.remaining_time().unwrap() > Duration::from_millis(90));
    }
}
