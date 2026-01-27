pub mod game_history;
pub mod perft;
pub mod picker;
pub mod score_table;
pub mod status;
pub mod window;

pub type Depth = u8;

use crate::{
    evaluation::{ValueScore, nnue::NeuralNetwork},
    position::Position,
    search::{
        game_history::GameHistory,
        picker::MovePicker,
        score_table::ScoreTable,
        status::{SearchStatus, SearchStatusValue},
        window::{FeedResult, Window},
    },
};
use primitive_enum::primitive_enum;
use std::time::{Duration, Instant};

const MATE_SCORE: ValueScore = ValueScore::MIN + 2;

primitive_enum! { NodeType u8;
    PVNode,
    AllNode,
    CutNode,
}

pub struct Searcher<'a> {
    history: &'a mut GameHistory,
    table: &'a mut ScoreTable,
    network: &'a mut NeuralNetwork,
    status: SearchStatus,
    initial: Instant,
    duration: Duration,
}

impl<'a> Searcher<'a> {
    pub fn new(
        history: &'a mut GameHistory,
        table: &'a mut ScoreTable,
        network: &'a mut NeuralNetwork,
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

    pub fn should_stop(&self) -> bool {
        let status = self.status.get();
        status == SearchStatusValue::Stopped
            || (status != SearchStatusValue::Pondering && self.initial.elapsed() >= self.duration)
    }

    pub fn quiesce(&mut self, position: &Position, ply: Depth, mut window: Window) -> (usize, ValueScore) {
        if self.should_stop() {
            return (1, window.best());
        }

        let is_check = position.is_check();

        if !is_check {
            let standing_pat = self.network.evaluate(position) * position.side_to_move().sign() as ValueScore;
            if matches!(window.feed(standing_pat, None), FeedResult::FailHigh) {
                return (1, window.best());
            }
        }

        let mut count = 0;
        let picker = MovePicker::new(position, !is_check, None, [None, None]);

        for mov in picker {
            let next_position = position.make_move(mov);
            let (nodes, score) = self.quiesce(&next_position, ply.saturating_add(1), window.reverse());
            count += nodes;
            if matches!(window.feed(-score, None), FeedResult::FailHigh) {
                break;
            }
        }

        if count == 0 {
            // We cannot detect stalemate here since our picker does not yield negative captures.
            let score = if is_check { MATE_SCORE + ply as ValueScore } else { window.best() };
            (1, score)
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
        if ply > 0 && (seen >= 3 || position.is_draw()) {
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
        let mut improved = false;
        let is_check = position.is_check();

        for mov in picker {
            let next_position = position.make_move(mov);
            // TODO: Determine children node type.

            self.history.push(&next_position, mov.is_reversible(position));
            let (nodes, score) =
                self.pvs(&next_position, depth - 1, ply.saturating_add(1), node_type, window.reverse());
            self.history.pop(next_position.side_to_move());

            count += nodes;

            match window.feed(-score, Some(mov)) {
                FeedResult::Improvement => {
                    node_type = NodeType::PVNode;
                    improved = true;
                }
                FeedResult::FailHigh => {
                    node_type = NodeType::CutNode;
                    break;
                }
                FeedResult::FailLow => {}
            }
        }

        if count == 0 {
            return (1, if is_check { MATE_SCORE + ply as ValueScore } else { 0 });
        }

        if node_type == NodeType::PVNode && !improved {
            node_type = NodeType::AllNode;
        }

        if !self.should_stop() {
            self.table
                .put(position, depth, ply, node_type, window.best(), window.best_move().unwrap());
        }

        (count, window.best())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evaluation::{MAX_POSITIONAL_WEIGHT, NNUE_PARAMS_BLOB, nnue::Parameters},
        position::fen::{Fen, START_POSITION},
    };
    use rstest::rstest;
    use std::{str::FromStr, thread::sleep};

    fn with_searcher(table_size: usize, body: impl Fn(&mut Searcher)) {
        let mut history = GameHistory::new(&Position::from_str(START_POSITION).unwrap());
        let mut table = ScoreTable::new_no_elems(table_size);
        let mut net = NeuralNetwork::new(Parameters::from_str(NNUE_PARAMS_BLOB).unwrap());
        let status = SearchStatus::new(SearchStatusValue::Searching);
        let mut searcher = Searcher::new(&mut history, &mut table, &mut net, status, Duration::from_hours(1));
        body(&mut searcher);
    }

    #[test]
    fn should_stop() {
        with_searcher(1, |searcher| {
            searcher.duration = Duration::from_millis(200);
            assert!(!searcher.should_stop());

            sleep(Duration::from_millis(200));
            assert!(searcher.should_stop());

            searcher.status.set(SearchStatusValue::Pondering);
            assert!(!searcher.should_stop());

            searcher.status.set(SearchStatusValue::Searching);
            assert!(searcher.should_stop());

            searcher.initial = Instant::now();
            assert!(!searcher.should_stop());

            searcher.status.set(SearchStatusValue::Stopped);
            assert!(searcher.should_stop());
        });
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
        with_searcher(1, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            let score =
                searcher.quiesce(&position, 0, Window::default()).1 * position.side_to_move().sign() as ValueScore;
            assert!((100 * material - score).abs() <= MAX_POSITIONAL_WEIGHT);
        });
    }

    #[rstest]
    #[case("k5R1/3n4/K7/4B3/8/8/8/8 b - - 0 1", 2)]
    #[case("k5RR/2bn4/K7/4B3/8/8/8/8 b - - 0 1", 4)]
    #[case("kb4RR/3n4/K7/4B3/8/8/8/8 w - - 1 2", 3)]
    fn quiesce_mates_through_captures(#[case] fen: Fen, #[case] plies: u8) {
        with_searcher(1, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            let score = searcher.quiesce(&position, 0, Window::default()).1;
            assert_eq!(score.abs(), (MATE_SCORE + plies as i16).abs());
        });
    }

    #[rstest]
    #[case("6k1/6p1/8/6KQ/1r6/q2b4/8/8 w - - 0 1", "h5e8")]
    #[case("5rk1/2Q3pp/p7/3Pp3/1P2P1P1/8/P4qPK/R6R b - - 2 30", "f2h4")]
    #[case("2k5/pp2n2Q/8/P2p4/6q1/P1p5/2P2P1P/5R1K b - - 2 22", "g4f3")]
    fn pvs_finds_perpetual(#[case] fen: Fen, #[case] mov: &str) {
        with_searcher(1, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            searcher.history.push(&position, true);
            searcher.history.push(&position, true);

            let score = searcher.pvs(&position, 5, 0, NodeType::PVNode, Window::default()).1;
            assert_eq!(score, 0);
            assert_eq!(searcher.table.hash_move(&position), Some(position.get_move_str(mov).unwrap()));
        });
    }
}
