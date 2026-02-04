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
            // We cannot detect stalemate here since the quiesce picker does not yield all moves when not in check.
            let score = if is_check { MATE_SCORE + ply as ValueScore } else { window.best() };
            (1, score)
        } else {
            (count, window.best())
        }
    }

    pub fn alphabeta(
        &mut self,
        position: &Position,
        mut depth: Depth,
        ply: Depth,
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
            && ply > 0
            && let Some((score, node_type)) = self.table.probe(position, depth, ply)
            && let Some(next) = window.feed_cache(score, node_type)
        {
            return (1, next);
        }

        // TODO: Implement killers.
        let picker = MovePicker::new(position, false, self.table.hash_move(position), [None, None]);

        let mut count = 0;
        let mut node_type = NodeType::AllNode;
        let is_check = position.is_check();

        if is_check {
            // Check extensions help tactical sequences and do not bloat the search too much.
            depth = depth.saturating_add(1);
        }

        for mov in picker {
            let next_position = position.make_move(mov);

            self.history.push(&next_position, mov.is_reversible(position));
            let (nodes, score) =
                self.alphabeta(&next_position, depth.saturating_sub(1), ply.saturating_add(1), window.reverse());
            self.history.pop(next_position.side_to_move());

            count += nodes;

            match window.feed(-score, Some(mov)) {
                FeedResult::Improvement => {
                    node_type = NodeType::PVNode;
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
    fn alphabeta_finds_perpetual(#[case] fen: Fen, #[case] mov: &str) {
        with_searcher(10_000, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            searcher.history.push(&position, true);
            searcher.history.push(&position, true);

            let score = searcher.alphabeta(&position, 5, 0, Window::default()).1;
            assert_eq!(score, 0);
            assert_eq!(searcher.table.hash_move(&position), Some(position.get_move_str(mov).unwrap()));
        });
    }

    #[rstest]
    #[case("5k2/8/6P1/6K1/8/8/8/8 w - - 0 1", "g5f6 f8e8 g6g7", 11)]
    #[case("6Q1/5K2/7k/8/8/8/8/8 b - - 0 4", "h6h5 g8g3 h5h6 g3h4", 4)]
    #[case("8/5pp1/6p1/5P1P/8/K7/8/k7 w - - 0 1", "f5f6 g7f6 h5h6", 6)]
    #[case("1k6/6R1/3K4/8/8/8/8/8 w - - 6 4", "d6c6 b8a8 c6b6 a8b8 g7g8", 5)]
    #[case("2k5/4B3/1K6/8/4B3/8/8/8 w - - 12 7", "e4f5 c8b8 e7d6 b8a8 f5e4", 3)]
    #[case("8/1p1b1k1r/p1pB2q1/2Pp3p/3PpPpQ/2P3P1/P5K1/4R3 w - - 0 45", "h4e7 f7g8 e7f8", 2)]
    #[case("1r4k1/2RQ1rp1/4p2p/pp1p4/8/7P/5PPK/4q3 w - - 0 38", "d7f7 g8h7 f7g7", 2)]
    #[case("2Q5/p3qkpp/5pb1/8/1p1R4/5P1P/PP4P1/7K b - - 0 35", "e7e1 h1h2 e1e5", 2)]
    #[case("5r1k/6pp/5q2/8/1Q6/8/6PP/2R3K1 b - - 2 46", "f6f2 g1h1 f2f1 c1f1 f8f1", 3)]
    #[case("r4rk1/pp2pp1p/6p1/1N2q3/2nN2Q1/8/P4PPP/4RK1R b - - 3 18", "c4d2 f1g1 e5e1", 2)]
    #[case("1rbk4/1p1p1Qpp/p2R4/P7/1P2r3/2q3P1/5K1P/3R4 w - - 0 28", "d6d7 c8d7 f7d7", 2)]
    #[case("7k/pp4p1/2p4p/6q1/8/2N1r2P/PPPQ2P1/R6K b - - 2 26", "e3h3 g2h3 g5d2", 2)]
    #[case("3r1k2/pp3p2/3pqp2/5N1p/2PP1Q2/6P1/P4P1P/6K1 w - - 5 32", "f4h6 f8e8 f5g7", 2)]
    #[case("2kr4/1pp2B2/p2p1R2/4p2p/4P3/3P3q/PPP1KBr1/3R4 w - - 1 26", "f7e6 h3e6 f6e6", 2)]
    #[case("r1bk1bNr/1pq2Qpp/2np4/4p3/2p1P3/8/PP3PPP/R4K1R w - - 4 17", "f7f8 d8d7 f8g7", 2)]
    #[case("6k1/1pRbb2p/4p1pP/p2p4/4qBP1/P5Q1/1P3K2/8 b - - 3 26", "e7h4 g3h4 e4f4", 3)]
    #[case("2q3k1/6pp/r2PR3/1p1n1p2/5B2/2P2B2/5PPP/4R1K1 b - - 0 38", "c8e6", 3)]
    #[case("5r2/kpb1q2p/p1R3pP/3Pp1P1/3pPp2/1Q1P3B/PP6/1K6 w - - 8 35", "d5d6 c7d6 b3b6 a7b8 c6d6", 4)]
    #[case("r5rk/1p1nq2p/3pnp1p/1pp5/4PP1P/P2PQ3/BPP2PR1/2K3R1 w - -", "g2g8 a8g8 g1g8 h8g8 f4f5", 4)]
    #[case("r5k1/2qn1pp1/bpp2n1p/p2pB3/8/2PQ3P/PPB1NPP1/R4RK1 b - - 0 18", "c7e5", 2)]
    #[case("8/p4pp1/1pp4p/2Pp4/1P1K1k2/P4P1P/8/8 w - - 0 34", "c5b6 a7b6 a3a4", 3)]
    #[case("8/8/3p1p2/p2PpP2/1p2P1rk/2P5/PP2B1K1/8 w - - 0 42", "e2g4 h4g4 c3c4", 8)]
    fn alphabeta_tactics(#[case] fen: Fen, #[case] moves: &str, #[case] depth: Depth) {
        with_searcher(10_000, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            let moves = moves.split_whitespace().collect::<Vec<_>>();
            searcher.alphabeta(&position, depth, 0, Window::default());
            let pv = searcher
                .table
                .pv_str(&position)
                .into_iter()
                .take(moves.len())
                .collect::<Vec<_>>();
            assert_eq!(pv, moves);
        });
    }
}
