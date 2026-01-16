use std::time::{Duration, Instant};

use primitive_enum::primitive_enum;

use crate::{
    core::position::Position,
    evaluation::{ValueScore, nnue::NeuralNetwork},
    search::{
        Depth, SearchStatus, SearchStatusValue,
        picker::MovePicker,
        pvs::{
            game_history::GameHistory,
            score_table::ScoreTable,
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
        Self { history, table, status, network, initial: Instant::now(), duration }
    }

    pub fn should_stop(&self) -> bool {
        self.status.get() == SearchStatusValue::Stopped || self.initial.elapsed() >= self.duration
    }

    pub fn quiesce(&mut self, position: &Position, ply: Depth, mut window: Window) -> (usize, ValueScore) {
        if self.should_stop() {
            return (1, window.best());
        }

        let standing_pat = self.network.evaluate(position) * position.side_to_move().sign() as ValueScore;
        if matches!(window.feed(standing_pat), FeedResult::FailHigh) {
            return (1, window.best());
        }

        let mut count = 0;
        let is_check = position.is_check();
        let picker = MovePicker::new(position, !is_check, None, [None, None]);

        for mov in picker {
            let next_position = position.make_move(mov);
            let (nodes, score) = self.quiesce(&next_position, ply.saturating_add(1), window.reverse());
            count += nodes;
            if matches!(window.feed(-score), FeedResult::FailHigh) {
                break;
            }
        }

        if count == 0 {
            (1, if is_check { MATE_SCORE + ply as ValueScore } else { window.best() })
        } else {
            (count, window.best())
        }
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
            return self.quiesce(position, ply, window);
        }

        let seen = self.history.seen(position);
        if seen >= 3 {
            return (1, 0);
        }

        if seen <= 1
            && let Some((score, node_type)) = self.table.probe(position, depth, ply)
            && let Some(next) = window.feed_cache(score, node_type)
        {
            return (1, next);
        }

        // TODO: Implement killers.
        let picker = MovePicker::new(position, false, self.table.hash_move(position), [None, None]);

        let mut count = 0;
        let mut best_move = None;
        let is_check = position.is_check();

        for mov in picker {
            let next_position = position.make_move(mov);
            // TODO: Determine children node type.

            self.history.push(&next_position, mov.is_reversible(position));
            let (nodes, score) =
                self.pvs(&next_position, depth - 1, ply.saturating_add(1), node_type, window.reverse());
            self.history.pop();

            count += nodes;

            match window.feed(-score) {
                FeedResult::Improvement => {
                    node_type = NodeType::PVNode;
                    best_move = Some(mov);
                }
                FeedResult::FailHigh => {
                    node_type = NodeType::CutNode;
                    best_move = Some(mov);
                    break;
                }
                FeedResult::FailLow => {}
            }
        }

        if count == 0 {
            return (1, if is_check { MATE_SCORE + ply as ValueScore } else { 0 });
        }

        if best_move.is_none() {
            node_type = NodeType::AllNode;
        }

        if !self.should_stop() {
            self.table.put(position, depth, ply, node_type, window.best(), best_move);
        }

        (count, window.best())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::position::fen::Fen,
        evaluation::{MAX_POSITIONAL_WEIGHT, NNUE_PARAMS_BLOB, nnue::Parameters},
    };
    use rstest::rstest;
    use std::{str::FromStr, thread::sleep};

    #[test]
    fn should_stop() {
        let status = SearchStatus::new(SearchStatusValue::Searching);
        let mut searcher = Searcher::new(
            GameHistory::default(),
            ScoreTable::new_no_elems(1),
            NeuralNetwork::new(Parameters::random()),
            status.clone(),
            Duration::from_millis(200),
        );

        assert!(!searcher.should_stop());

        sleep(Duration::from_millis(200));
        assert!(searcher.should_stop());

        searcher.initial = Instant::now();
        assert!(!searcher.should_stop());

        status.set(SearchStatusValue::Stopped);
        assert!(searcher.should_stop());
    }

    #[rstest]
    #[case("rn1qkb1r/pppb1ppp/4pn2/8/Q1pP4/2N2N2/PP2PPPP/R1B1KB1R w KQkq - 4 6", 0)]
    #[case("r2qk2r/pppb1ppp/2n1pn2/8/QbpPP3/2N2N2/PP2BPPP/R1B2RK1 b kq - 4 8", 1)]
    #[case("3q2k1/3bbpp1/p1r1p2p/1p1pP3/3P4/1P4P1/P1R2P1P/2Q1NBK1 w - - 9 31", 3)]
    #[case("6k1/2qbbpp1/p1r1p2p/1p1pP3/3P4/1P3NP1/P1R2PBP/2Q3K1 b - - 12 32", -5)]
    #[case("3b2k1/2qb1pp1/pr2p2p/1p1pP3/3P4/1P3NPP/P1R2PB1/2Q3K1 w - - 1 34", 7)]
    #[case("rnbq1rk1/ppBpppbp/5np1/8/3P4/5N2/PPP1PPPP/RN1QKB1R b KQ - 0 5", -2)]
    #[case("r1b1r1k1/1pq2pbp/p1n2np1/3pp3/3P4/P1NBPN2/1PP2PPP/R2Q1RK1 w - - 0 11", -2)]
    #[case("3rr1k1/1p3p1p/p2q2p1/5b2/2P5/P1N1PB2/1b3PPP/1R1QR1K1 b - - 0 19", -2)]
    #[case("8/6pk/p7/5q2/8/6Q1/3rrNPP/R5K1 b - - 5 30", -5)]
    #[case("r1bqkb1r/ppp1pppp/1n3n2/3P4/8/2N2Q2/PPPPBPPP/R1B1K1NR b KQkq - 6 5", 0)]
    fn quiesce_captures(#[case] fen: Fen, #[case] material: i16) {
        let mut searcher = Searcher::new(
            GameHistory::default(),
            ScoreTable::new_no_elems(1),
            NeuralNetwork::new(Parameters::from_str(NNUE_PARAMS_BLOB).unwrap()),
            SearchStatus::new(SearchStatusValue::Searching),
            Duration::from_hours(1),
        );

        let position = Position::try_from(fen.clone()).unwrap();
        let score = searcher.quiesce(&position, 0, Window::default()).1 * position.side_to_move().sign() as ValueScore;
        assert!((100 * material - score).abs() <= MAX_POSITIONAL_WEIGHT);
    }
}
