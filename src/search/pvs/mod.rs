use std::time::{Duration, Instant};

use primitive_enum::primitive_enum;

use crate::{
    core::{moves::Move, position::Position},
    evaluation::{ValueScore, nnue::NeuralNetwork},
    search::{
        Depth, SearchStatus, SearchStatusValue,
        picker::MovePicker,
        pvs::{
            game_history::GameHistory,
            score_table::{Entry, ScoreTable},
            window::{FeedResult, Window},
        },
    },
};

pub mod game_history;
pub mod score_table;
pub mod window;

const MATE_SCORE: ValueScore = ValueScore::MIN + 1;

primitive_enum! { NodeType u8;
    PVNode,
    AllNode,
    CutNode,
}

pub struct Searcher {
    history: GameHistory,
    table: ScoreTable,
    network: NeuralNetwork,
    status: SearchStatus,
    initial: Instant,
    duration: Duration,
}

impl Searcher {
    pub fn new(
        history: GameHistory,
        table: ScoreTable,
        network: NeuralNetwork,
        status: SearchStatus,
        duration: Duration,
    ) -> Self {
        Self {
            history,
            table,
            status,
            network,
            initial: Instant::now(),
            duration,
        }
    }

    fn should_stop(&self) -> bool {
        self.status.get() == SearchStatusValue::Stopped || self.initial.elapsed() >= self.duration
    }

    pub fn pvs(
        &mut self,
        position: &Position,
        depth: Depth,
        ply: Depth,
        mut node_type: NodeType,
        mut window: Window,
    ) -> (usize, ValueScore) {
        if self.should_stop() {
            return (1, window.best());
        }

        if depth == 0 {
            // TODO: Implement quiesce.
            return (1, self.network.evaluate(position));
        }

        if self.history.seen(position) >= 3 {
            return (1, 0);
        }

        if let Some(Entry { score, node_type, .. }) = self.table.probe(position, depth, ply) {
            if let Some(_) = window.feed_cache(score, node_type) {
                return (1, score);
            }
        }

        // TODO: Implement killers.
        let picker = MovePicker::new(position, false, self.table.hash_move(position), [None, None]);

        let mut count = 0;
        let mut best_move = Move::blank();
        let initial_best = window.best();

        for mov in picker {
            let next_position = position.make_move(mov);
            // TODO: Determine children node type.

            self.history.push(&next_position, mov.is_reversible(position));
            let (nodes, score) = self.pvs(&next_position, depth - 1, ply + 1, node_type, window.reverse());
            self.history.pop();

            count += nodes;

            match window.feed(-score) {
                FeedResult::Improvement => {
                    node_type = NodeType::PVNode;
                    best_move = mov;
                }
                FeedResult::FailHigh => {
                    node_type = NodeType::CutNode;
                    break;
                }
                FeedResult::FailLow => {}
            }
        }

        if count == 0 {
            return (1, MATE_SCORE + ply as ValueScore);
        }

        if initial_best == window.best() {
            node_type = NodeType::AllNode;
        }

        if !self.should_stop() {
            self.table
                .put(position, depth, ply, node_type, window.best(), best_move);
        }

        (count, window.best())
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn should_stop() {
        let status = SearchStatus::new(SearchStatusValue::Searching);
        let mut seacher = Searcher::new(
            GameHistory::default(),
            ScoreTable::new_no_elems(1),
            NeuralNetwork::blank(),
            status.clone(),
            Duration::from_secs(1),
        );

        assert!(!seacher.should_stop());

        sleep(Duration::from_secs(1));
        assert!(seacher.should_stop());

        seacher.initial = Instant::now();
        assert!(!seacher.should_stop());

        status.set(SearchStatusValue::Stopped);
        assert!(seacher.should_stop());
    }
}
