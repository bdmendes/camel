mod pvs;

use crate::{
    evaluation::{Score, MATE_LOWER, MATE_UPPER},
    position::{moves::Move, zobrist::ZobristHash, Position},
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

pub use self::pvs::pvs;

pub type Depth = u8;

const MAX_TABLE_SIZE: usize = 64_000_000;
pub const MAX_MATE_SCORE_DIFF: Score = 300;

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
    pub killer_moves: HashMap<Depth, [Option<Move>; 2]>,
    pub hash_move: HashMap<ZobristHash, (Move, Depth)>,
    pub transposition_table: HashMap<ZobristHash, TranspositionEntry>,
    pub branch_history: Vec<ZobristHash>,
    pub initial_instant: std::time::Instant,
    pub duration: Option<std::time::Duration>,
    pub stop_now: Option<Arc<AtomicBool>>,
}

impl SearchMemo {
    pub fn new(
        duration: Option<std::time::Duration>,
        stop_now: Option<Arc<AtomicBool>>,
        game_history: Option<Vec<ZobristHash>>,
    ) -> Self {
        Self {
            killer_moves: HashMap::new(),
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

    fn put_killer_move(&mut self, mov: &Move, depth: Depth) {
        let killer_moves = self.killer_moves.entry(depth).or_insert([None, None]);
        if killer_moves[0] == None {
            killer_moves[0] = Some(*mov);
        } else if killer_moves[1] == None {
            killer_moves[1] = Some(*mov);
        } else if Some(*mov) == killer_moves[0] {
            killer_moves[1] = Some(*mov);
        } else {
            killer_moves[0] = Some(*mov);
        }
    }

    fn get_killer_moves(&mut self, depth: Depth) -> [Option<Move>; 2] {
        *self.killer_moves.entry(depth).or_insert([None, None])
    }

    fn is_killer_move(mov: &Move, killer_moves: [Option<Move>; 2]) -> bool {
        killer_moves[0] == Some(*mov) || killer_moves[1] == Some(*mov)
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

    fn cleanup_tables(&mut self) {
        if self.killer_moves.len() > MAX_TABLE_SIZE {
            self.killer_moves.clear();
        }
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
        if distance_to_mate < (depth + 1) as Score {
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
