mod pvs;

use crate::{
    evaluation::{Score, MATE_LOWER, MATE_UPPER},
    position::{moves::Move, zobrist::ZobristHash, Color, Position},
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use self::pvs::pvs;

pub type Depth = u8;

const MAX_TABLE_SIZE: usize = 64_000_000;
const MAX_MATE_SCORE_DIFF: Score = 300;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Node {
    PVNode,  // exact score
    CutNode, // lower bound
    AllNode, // upper bound
}

pub struct TranspositionEntry {
    score: Score,
    depth: Depth,
    node: Node,
    best_move: Option<Move>,
}

pub struct SearchMemo {
    pub history_table: [[[Score; 64]; 64]; 2], // [color][from][to]
    pub hash_move: HashMap<ZobristHash, (Move, Depth)>,
    pub transposition_table: HashMap<ZobristHash, TranspositionEntry>,
    pub branch_history: Vec<ZobristHash>,
    pub initial_instant: std::time::Instant,
    pub duration: Option<std::time::Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl SearchMemo {
    fn new(
        duration: Option<std::time::Duration>,
        stop_now: Option<Arc<AtomicBool>>,
        game_history: Option<Vec<ZobristHash>>,
    ) -> Self {
        Self {
            history_table: [[[0; 64]; 64]; 2],
            hash_move: HashMap::new(),
            transposition_table: HashMap::new(),
            branch_history: game_history.unwrap_or_default(),
            initial_instant: std::time::Instant::now(),
            duration,
            stop_now,
        }
    }

    fn get_principal_variation(&self, position: &Position, depth: Depth) -> Vec<Move> {
        let mut principal_variation = Vec::new();
        let mut current_position = position.clone();
        let mut current_depth = depth;
        while current_depth > 0 {
            let entry = self.hash_move.get(&current_position.zobrist_hash());
            if entry.is_none() {
                break;
            }

            let mov = entry.unwrap().0;
            principal_variation.push(mov);
            current_position = current_position.make_move(&mov);
            current_depth -= 1;
        }
        principal_variation
    }

    fn put_hash_move(&mut self, zobrist_hash: ZobristHash, mov: &Move, depth: Depth) {
        let (hash_mov, hash_depth) = self.hash_move.entry(zobrist_hash).or_insert((*mov, 0));
        if depth <= *hash_depth {
            return;
        }
        *hash_depth = depth;
        *hash_mov = *mov;
    }

    fn is_hash_move(mov: &Move, hash_move: Option<&Move>) -> bool {
        hash_move == Some(mov)
    }

    fn put_transposition_table(
        &mut self,
        zobrist_hash: ZobristHash,
        depth: Depth,
        mov: Option<Move>,
        score: Score,
        node: Node,
    ) {
        if let Some(entry) = self.transposition_table.get_mut(&zobrist_hash) {
            if depth < entry.depth || (depth == entry.depth && entry.node == Node::PVNode) {
                return;
            }
            entry.depth = depth;
            entry.best_move = mov;
            entry.score = score;
            entry.node = node;
            return;
        }

        self.transposition_table
            .insert(zobrist_hash, TranspositionEntry { score, depth, node, best_move: mov });
    }

    fn get_transposition_table(
        &mut self,
        zobrist_hash: ZobristHash,
        depth: Depth,
    ) -> Option<&TranspositionEntry> {
        if let Some(tt_entry) = self.transposition_table.get(&zobrist_hash) {
            if depth <= tt_entry.depth {
                return Some(tt_entry);
            }
        }

        None
    }

    fn visit_position(&mut self, zobrist_hash: ZobristHash) {
        self.branch_history.push(zobrist_hash);
    }

    fn leave_position(&mut self) {
        self.branch_history.pop();
    }

    fn put_history_table(&mut self, mov: &Move, color: Color, depth: Depth) {
        let from = mov.from.index;
        let to = mov.to.index;
        let depth = std::cmp::min(0, depth);
        self.history_table[color as usize][from][to] = (depth * depth) as Score;
    }

    fn get_history_value(&self, mov: &Move, color: Color) -> Score {
        let from = mov.from.index;
        let to = mov.to.index;
        self.history_table[color as usize][from][to]
    }

    fn cleanup_tables(&mut self) {
        if self.hash_move.len() > MAX_TABLE_SIZE {
            self.hash_move.clear();
        }
        if self.transposition_table.len() > MAX_TABLE_SIZE {
            self.transposition_table.clear();
        }
    }

    fn should_stop_search(&self) -> bool {
        if let Some(duration) = self.duration {
            if self.initial_instant.elapsed() > duration {
                return true;
            }
        }

        if let Some(ref stop_now) = self.stop_now {
            if stop_now.load(std::sync::atomic::Ordering::Relaxed) {
                return true;
            }
        }

        false
    }
}

fn print_iterative_info(
    position: &Position,
    memo: &SearchMemo,
    depth: Depth,
    score: Score,
    nodes: usize,
    time: Duration,
) {
    let distance_to_mate = MATE_UPPER - score.abs();
    print!(
        "info depth {} score {} time {} nodes {} nps {}",
        depth,
        if distance_to_mate < MAX_MATE_SCORE_DIFF {
            format!(
                "mate {}{}",
                if score < 0 && distance_to_mate > 0 { "-" } else { "" },
                (distance_to_mate + 1) / 2
            )
        } else {
            format!("cp {}", score)
        },
        time.as_millis(),
        nodes,
        (nodes as f64 / time.as_secs_f64()).floor()
    );
    let pv = memo.get_principal_variation(position, depth);
    if !pv.is_empty() {
        print!(" pv");
        for mov in pv {
            print!(" {}", mov);
        }
    }
    println!();
}

pub fn search_iterative_deep(
    position: &Position,
    depth: Option<Depth>,
    duration: Option<std::time::Duration>,
    stop_now: Option<Arc<AtomicBool>>,
    game_history: Option<Vec<ZobristHash>>,
) -> (Option<Move>, Score, usize) {
    const MAX_ITERATIVE_DEPTH: Depth = 25;
    let max_depth = depth.unwrap_or(MAX_ITERATIVE_DEPTH);
    let mut memo = SearchMemo::new(duration, stop_now.clone(), game_history);

    // First guaranteed search
    let start = Instant::now();
    let (mut mov, mut score, mut nodes) = pvs(position, 1, MATE_LOWER, MATE_UPPER, &mut memo, 1);
    print_iterative_info(position, &memo, 1, score, nodes, start.elapsed());

    // Time constrained iterative deepening
    for ply in 2..=max_depth {
        let start = Instant::now();
        let (new_mov, new_score, new_nodes) =
            pvs(position, ply, MATE_LOWER, MATE_UPPER, &mut memo, ply);

        if memo.should_stop_search() {
            break;
        }

        print_iterative_info(position, &memo, ply, score, nodes, start.elapsed());

        mov = new_mov;
        score = new_score;
        nodes = new_nodes;

        memo.cleanup_tables();
    }

    println!(
        "bestmove {}",
        if mov.is_some() { mov.unwrap().to_string() } else { "(none)".to_string() }
    );
    return (mov, score, nodes);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_search(
        fen: &str,
        depth: Depth,
        expected_move: &str,
        expected_lower_score: Option<Score>,
        expected_upper_score: Option<Score>,
        test_regular: bool,
    ) {
        let position = Position::from_fen(fen).unwrap();

        let now = std::time::Instant::now();
        let (reg_mov, reg_score, reg_nodes) =
            search_iterative_deep(&position, Some(depth), None, None, None);
        let elapsed = now.elapsed().as_millis();
        println!("[iterative] {}: {} nodes in {} ms at depth {}\n", fen, reg_nodes, elapsed, depth);

        if !test_regular {
            return;
        }

        let now = std::time::Instant::now();
        let (iter_mov, iter_score, iter_nodes) = pvs(
            &position,
            depth,
            MATE_LOWER,
            MATE_UPPER,
            &mut SearchMemo::new(None, None, None),
            depth,
        );
        let elapsed = now.elapsed().as_millis();
        println!("[regular] {}: {} nodes in {} ms at depth {}", fen, iter_nodes, elapsed, depth);

        assert_eq!(reg_mov.unwrap().to_string(), expected_move);
        if let Some(expected_lower_score) = expected_lower_score {
            assert!(reg_score >= expected_lower_score - MAX_MATE_SCORE_DIFF);
        }
        if let Some(expected_upper_score) = expected_upper_score {
            assert!(reg_score <= expected_upper_score + MAX_MATE_SCORE_DIFF);
        }

        assert_eq!(iter_mov.unwrap().to_string(), expected_move);
        if let Some(expected_lower_score) = expected_lower_score {
            assert!(iter_score >= expected_lower_score - MAX_MATE_SCORE_DIFF);
        }
        if let Some(expected_upper_score) = expected_upper_score {
            assert!(iter_score <= expected_upper_score + MAX_MATE_SCORE_DIFF);
        }
    }

    #[test]
    fn search_mate_in_1() {
        test_search("3k4/6R1/8/7R/8/8/8/4k3 w - - 0 1", 5, "h5h8", Some(MATE_UPPER), None, false);
    }

    #[test]
    fn search_forced_capture() {
        test_search(
            "r2N2k1/p1R3pp/8/1pP5/1b2p1bP/4P3/4BPP1/3K3R b - - 0 24",
            4,
            "a8d8",
            None,
            None,
            false,
        );
    }

    #[test]
    fn search_trapped_knight() {
        test_search(
            "r2B1rk1/pp1n1pp1/2p1p1p1/8/3P4/5N1P/PPP3P1/R2nR1K1 w - - 0 15",
            4,
            "d8e7",
            None,
            None,
            false,
        );
    }

    #[test]
    fn search_check_combination() {
        test_search(
            "4r2k/1p4p1/3P3p/P1P5/4b3/1Bq5/6PP/3QR1K1 b - - 11 41",
            4,
            "c3c5",
            None,
            None,
            false,
        );
    }

    #[test]
    fn search_endgame_opposition() {
        test_search("5k2/8/6K1/5P2/8/8/8/8 w - - 0 20", 10, "g6f6", None, None, true);
    }
}
