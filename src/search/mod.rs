pub mod game_history;
pub mod heuristics;
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
        heuristics::maybe_zug,
        picker::MovePicker,
        score_table::ScoreTable,
        status::{SearchStatus, SearchStatusValue},
        window::{FeedResult, Window},
    },
};
use primitive_enum::primitive_enum;
use std::time::{Duration, Instant};

const MATE_SCORE: ValueScore = ValueScore::MIN + 2;
const NULL_MOVE_MIN_DEPTH: Depth = 5;
const NULL_MOVE_REDUCTION: Depth = 3;

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

        if ply > 0
            && let Some((score, node_type)) = self.table.probe(position, depth, ply)
            && let Some(next) = window.feed_cache(score, node_type)
        {
            return (1, next);
        }

        let is_check = position.is_check();

        if ply > 0 && !is_check && depth > NULL_MOVE_MIN_DEPTH && !maybe_zug(position) {
            let next = position.flipped_side();
            let (nodes, score) =
                self.alphabeta(&next, depth - NULL_MOVE_REDUCTION, ply.saturating_add(1), window.reverse_null());
            if window.cuts_off(-score) {
                return (nodes + 1, -score);
            }
        }

        let picker = MovePicker::new(position, false, self.table.hash_move(position), [None, None]);

        let mut count = 0;
        let mut node_type = NodeType::AllNode;

        if is_check {
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
    #[case("2k5/4B3/1K6/8/4B3/8/8/8 w - - 12 7", "e4f5 c8b8 e7d6 b8a8 f5e4", 4)]
    #[case("8/1p1b1k1r/p1pB2q1/2Pp3p/3PpPpQ/2P3P1/P5K1/4R3 w - - 0 45", "h4e7 f7g8 e7f8", 2)]
    #[case("1r4k1/2RQ1rp1/4p2p/pp1p4/8/7P/5PPK/4q3 w - - 0 38", "d7f7 g8h7 f7g7", 2)]
    #[case("2Q5/p3qkpp/5pb1/8/1p1R4/5P1P/PP4P1/7K b - - 0 35", "e7e1 h1h2 e1e5", 2)]
    #[case("5r1k/6pp/5q2/8/1Q6/8/6PP/2R3K1 b - - 2 46", "f6f2 g1h1 f2f1 c1f1 f8f1", 4)]
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
    #[case("5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - - 0 1", "e3g3", 2)]
    #[case("r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - - 0 1", "h6h7 h8h7 h5g6", 2)]
    #[case("7k/p7/1R5K/6r1/6p1/6P1/8/8 w - -", "b6b7", 3)]
    #[case("rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq - 0 1", "g4e3", 3)]
    #[case("r4q1k/p2bR1rp/2p2Q1N/5p2/5p2/2P5/PP3PPP/R5K1 w - - 0 1", "e7f7", 2)]
    #[case("2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - -", "h4h7", 2)]
    #[case("5rk1/pp4p1/2n1p2p/2Npq3/2p5/6P1/P3P1BP/R4Q1K w - -", "f1f8 g8f8 c5d7", 2)]
    #[case("r4rk1/ppp2ppp/2n5/2bqp3/8/P2PB3/1PP1NPPP/R2Q1RK1 w - -", "e2c3", 2)]
    #[case("1k5r/pppbn1pp/4q1r1/1P3p2/2NPp3/1QP5/P4PPP/R1B1R1K1 w - - 0 1", "c4e5", 2)]
    #[case("R7/P4k2/8/8/8/8/r7/6K1 w - - 0 1", "a8h8 a2a1 g1f2 a1a2", 7)]
    #[case("r1b2rk1/ppbn1ppp/4p3/1QP4q/3P4/N4N2/5PPP/R1B2RK1 w - - 0 1", "c5c6", 2)]
    #[case("r2qkb1r/1ppb1ppp/p7/4p3/P1Q1P3/2P5/5PPP/R1B2KNR b kq - 0 1", "d7b5 c4b5 a6b5", 3)]
    #[case("5rk1/1b3p1p/pp3p2/3n1N2/1P6/P1qB1PP1/3Q3P/4R1K1 w - - 0 1", "d2h6 c3e1 d3f1 e1e3 f5e3", 5)]
    #[case("1r1r2k1/4pp1p/2p1b1p1/p3R3/RqBP4/4P3/1PQ2PPP/6K1 b - - 0 1", "b4e1 c4f1 e6b3", 4)]
    #[case("r2q2k1/pp1rbppp/4pn2/2P5/1P3B2/6P1/P3QPBP/1R3RK1 w - - 0 1", "c5c6 b7c6 g2c6", 4)]
    #[case("1r3r2/4q1kp/b1pp2p1/5p2/pPn1N3/6P1/P3PPBP/2QRR1K1 w - - 0 1", "e4d6 c4d6 c1c6", 4)]
    #[case("6k1/p4p1p/1p3np1/2q5/4p3/4P1N1/PP3PPP/3Q2K1 w - - 0 1", "d1d8 g8g7 d8f6 g7f6 g3e4", 4)]
    #[case("7k/1b1r2p1/p6p/1p2qN2/3bP3/3Q4/P5PP/1B1R3K b - - 0 1", "d4g1", 2)]
    #[case("r3r2k/2R3pp/pp1q1p2/8/3P3R/7P/PP3PP1/3Q2K1 w - - 0 1", "h4h7 h8h7 d1h5 h7g8 h5f7", 4)]
    #[case("3r4/2p1rk2/1pQq1pp1/7p/1P1P4/P4P2/6PP/R1R3K1 b - - 0 1", "e7e1", 2)]
    #[case("2r5/2rk2pp/1pn1pb2/pN1p4/P2P4/1N2B3/nPR1KPPP/3R4 b - - 0 1", "c6d4", 2)]
    #[case("r1br2k1/pp2bppp/2nppn2/8/2P1PB2/2N2P2/PqN1B1PP/R2Q1R1K w - - 0 1", "c3a4 b2a1", 2)]
    #[case("3rb1k1/pq3pbp/4n1p1/3p4/2N5/2P2QB1/PP3PPP/1B1R2K1 b - - 0 1", "d5c4 d1d8", 2)]
    #[case("7k/2p1b1pp/8/1p2P3/1P3r2/2P3Q1/1P5P/R4qBK b - - 0 1", "f1a1", 2)]
    #[case("r1bqr1k1/pp1nb1p1/4p2p/3p1p2/3P4/P1N1PNP1/1PQ2PP1/3RKB1R w K - 0 1", "c3b5", 6)]
    #[case("r1b2rk1/pp2bppp/2n1pn2/q5B1/2BP4/2N2N2/PP2QPPP/2R2RK1 b - - 0 1", "c6d4 f3d4 a5g5", 6)]
    #[case("k4r2/1R4pb/1pQp1n1p/3P4/5p1P/3P2P1/r1q1R2K/8 w - - 0 1", "b7b6 c2c6 e2a2 c6a4 a2a4", 4)]
    #[case("r1bq1r2/pp4k1/4p2p/3pPp1Q/3N1R1P/2PB4/6P1/6K1 w - - 0 1", "f4g4 d8g5 h4g5", 4)]
    #[case("6k1/6p1/p7/3Pn3/5p2/4rBqP/P4RP1/5QK1 b - - 0 1", "e3e1", 2)]
    #[case("r3r1k1/pp1q1pp1/4b1p1/3p2B1/3Q1R2/8/PPP3PP/4R1K1 w - - 0 1", "d4g7 g8g7 g5f6", 6)]
    #[case("r3q1kr/ppp5/3p2pQ/8/3PP1b1/5R2/PPP3P1/5RK1 w - - 0 1", "f3f8 e8f8 f1f8 a8f8 h6g6", 4)]
    #[case("8/8/2R5/1p2qp1k/1P2r3/2PQ2P1/5K2/8 w - - 0 1", "d3d1 h5g5 d1d2", 4)]
    #[case("r1b2rk1/2p1qnbp/p1pp2p1/5p2/2PQP3/1PN2N1P/PB3PP1/3R1RK1 w - - 0 1", "c3d5", 2)]
    #[case("6r1/3Pn1qk/p1p1P1rp/2Q2p2/2P5/1P4P1/P3R2P/5RK1 b - - 0 1", "g6g3 g1h1", 4)]
    #[case("r1brnbk1/ppq2pp1/4p2p/4N3/3P4/P1PB1Q2/3B1PPP/R3R1K1 w - - 0 1", "e5f7", 3)]
    #[case("1r1r1qk1/p2n1p1p/bp1Pn1pQ/2pNp3/2P2P1N/1P5B/P6P/3R1RK1 w - - 0 1", "d5e7", 4)]
    #[case("1k1r2r1/ppq5/1bp4p/3pQ3/8/2P2N2/PP4P1/R4R1K b - - 0 1", "c7e5 f3e5 g8g5", 5)]
    #[case("3r2k1/p2q4/1p4p1/3rRp1p/5P1P/6PK/P3R3/3Q4 w - - 0 1", "e5d5 d7d5 e2e8", 3)]
    #[case("6k1/5ppp/1q6/2b5/8/2R1pPP1/1P2Q2P/7K w - - 0 1", "e2e3", 3)]
    #[case("2kr3r/pppq1ppp/3p1n2/bQ2p3/1n1PP3/1PN1BN1P/1PP2PP1/2KR3R b - - 0 1", "b4a2", 2)]
    #[case("2kr3r/pp1q1ppp/5n2/1Nb5/2Pp1B2/7Q/P4PPP/1R3RK1 w - - 0 1", "b5a7 c5a7 h3a3", 9)]
    fn alphabeta_iter_tactics(#[case] fen: Fen, #[case] moves: &str, #[case] depth: Depth) {
        with_searcher(1_000_000, |searcher| {
            let position = Position::try_from(fen.clone()).unwrap();
            let moves = moves.split_whitespace().collect::<Vec<_>>();
            for d in 1..=depth {
                searcher.alphabeta(&position, d, 0, Window::default());
            }
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
