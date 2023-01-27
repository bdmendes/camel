use crate::{
    evaluation::{
        evaluate_game_over, evaluate_move, evaluate_position, Score,
        MATE_LOWER, MATE_UPPER,
    },
    position::{moves::Move, zobrist::ZobristHash, Color, Position},
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

pub type Depth = i8;

const MAX_ITERATIVE_DEPTH: Depth = 40;
const MAX_TABLE_SIZE: usize = 1_000_000;
const MAX_QS_DEPTH: Depth = 5;
const MAX_DURATION_PER_MOVE: std::time::Duration =
    std::time::Duration::from_secs(5 * 60);

struct SearchMemo {
    pub killer_moves: HashMap<Depth, [Option<Move>; 2]>,
    pub hash_move: HashMap<ZobristHash, (Move, Depth)>,
    pub transposition_table: HashMap<ZobristHash, (Option<Move>, Score, Depth)>,
    pub initial_instant: std::time::Instant,
    pub duration: std::time::Duration,
}

impl SearchMemo {
    fn new(duration: Option<std::time::Duration>) -> Self {
        Self {
            killer_moves: HashMap::new(),
            hash_move: HashMap::new(),
            transposition_table: HashMap::new(),
            initial_instant: std::time::Instant::now(),
            duration: duration.unwrap_or(MAX_DURATION_PER_MOVE),
        }
    }

    fn put_killer_move(&mut self, mov: Move, depth: Depth) {
        let killer_moves =
            self.killer_moves.entry(depth).or_insert([None, None]);
        if killer_moves[0] == None {
            killer_moves[0] = Some(mov);
        } else if killer_moves[1] == None {
            killer_moves[1] = Some(mov);
        } else if Some(mov) == killer_moves[0] {
            killer_moves[1] = Some(mov);
        } else {
            killer_moves[0] = Some(mov);
        }
    }

    fn get_killer_moves(&mut self, depth: Depth) -> [Option<Move>; 2] {
        *self.killer_moves.entry(depth).or_insert([None, None])
    }

    fn is_killer_move(mov: Move, killer_moves: [Option<Move>; 2]) -> bool {
        killer_moves[0] == Some(mov) || killer_moves[1] == Some(mov)
    }

    fn put_hash_move(
        &mut self,
        zobrist_hash: ZobristHash,
        mov: Move,
        depth: Depth,
    ) {
        let (hash_mov, hash_depth) =
            self.hash_move.entry(zobrist_hash).or_insert((mov, 0));
        if depth <= *hash_depth {
            return;
        }
        *hash_depth = depth;
        *hash_mov = mov;
    }

    fn is_hash_move(mov: Move, hash_move: Option<Move>) -> bool {
        hash_move == Some(mov)
    }

    fn put_transposition_table(
        &mut self,
        zobrist_hash: ZobristHash,
        depth: Depth,
        mov: Option<Move>,
        score: Score,
    ) {
        let (transp_mov, transp_score, transp_depth) = self
            .transposition_table
            .entry(zobrist_hash)
            .or_insert((mov, score, 0));
        if depth <= *transp_depth {
            return;
        }
        *transp_depth = depth;
        *transp_mov = mov;
        *transp_score = score;
    }

    fn get_transposition_table(
        &mut self,
        zobrist_hash: ZobristHash,
        depth: Depth,
    ) -> Option<(Option<Move>, Score)> {
        if let Some((mov, score, transp_depth)) =
            self.transposition_table.get(&zobrist_hash)
        {
            if depth <= *transp_depth {
                return Some((*mov, *score));
            }
        }
        None
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
}

fn should_stop_search(
    memo: &SearchMemo,
    stop_now: Option<Arc<AtomicBool>>,
) -> bool {
    if memo.initial_instant.elapsed() > memo.duration {
        return true;
    }
    if let Some(ref stop_now) = stop_now {
        if stop_now.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }
    }
    false
}

fn alphabeta_quiet(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    memo: &SearchMemo,
    stop_now: Option<Arc<AtomicBool>>,
) -> (Score, usize) {
    // Check if the search should be stopped
    if should_stop_search(memo, stop_now.clone()) {
        return (alpha, 1);
    }

    // Calculate static evaluation and return if quiescence search depth is reached
    let color_cof = match position.info.to_move {
        Color::White => 1,
        Color::Black => -1,
    };
    let static_evaluation = color_cof * evaluate_position(position);
    if depth == 0 {
        return (static_evaluation, 1);
    }

    // Alpha-beta prune based on static evaluation
    if static_evaluation >= beta {
        return (beta, 1);
    }

    // Generate and sort non-quiet moves
    let mut moves = position.legal_moves(true);
    moves.sort_unstable_by(|a, b| {
        let a_value = evaluate_move(*a, &position, false, false);
        let b_value = evaluate_move(*b, &position, false, false);
        b_value.cmp(&a_value)
    });

    // Evaluate statically if only quiet moves are left
    if moves.is_empty() {
        return (static_evaluation, 1);
    }

    // Set lower bound to alpha ("standing pat")
    if static_evaluation > alpha {
        alpha = static_evaluation;
    }

    // Search moves
    let mut count = 0;
    for mov in moves {
        let new_position = position.make_move(mov);
        let (score, nodes) = alphabeta_quiet(
            &new_position,
            depth - 1,
            -beta,
            -alpha,
            memo,
            stop_now.clone(),
        );
        let score = -score;
        count += nodes;

        if score > alpha {
            alpha = score;
            if alpha >= beta {
                break;
            }
        }
    }

    (alpha, count)
}

fn alphabeta_memo(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    memo: &mut SearchMemo,
    stop_now: Option<Arc<AtomicBool>>,
) -> (Option<Move>, Score, usize) {
    // Check if the search should be stopped
    if should_stop_search(memo, stop_now.clone()) {
        return (None, alpha, 1);
    }

    // Check for transposition table hit
    let zobrist_hash = position.to_zobrist_hash();
    if let Some(res) = memo.get_transposition_table(zobrist_hash, depth) {
        return (res.0, res.1, 1);
    }

    // Cleanup tables if they get too big
    memo.cleanup_tables();

    // Enter quiescence search if depth is 0
    if depth == 0 {
        let (score, nodes) = alphabeta_quiet(
            position,
            MAX_QS_DEPTH,
            alpha,
            beta,
            memo,
            stop_now,
        );
        return (None, score, nodes);
    }

    // When game is over, do not search
    let mut moves = position.legal_moves(false);
    if let Some(score) = evaluate_game_over(position, &moves) {
        return (None, score + position.info.full_move_number as Score, 1);
    }

    // Sort moves by heuristic value + killer move + hash move
    let killer_moves = memo.get_killer_moves(depth);
    let hash_move = memo.hash_move.get(&zobrist_hash).map(|(mov, _)| *mov);
    moves.sort_unstable_by(|a, b| {
        let a_value = evaluate_move(
            *a,
            &position,
            SearchMemo::is_killer_move(*a, killer_moves),
            SearchMemo::is_hash_move(*a, hash_move),
        );
        let b_value = evaluate_move(
            *b,
            &position,
            SearchMemo::is_killer_move(*b, killer_moves),
            SearchMemo::is_hash_move(*b, hash_move),
        );
        b_value.cmp(&a_value)
    });

    // Search moves
    let mut best_move = moves[0];
    let mut count = 0;
    for mov in moves {
        let new_position = position.make_move(mov);
        let (_, score, nodes) = alphabeta_memo(
            &new_position,
            depth - 1,
            -beta,
            -alpha,
            memo,
            stop_now.clone(),
        );
        let score = -score;
        count += nodes;

        if score > alpha {
            best_move = mov;
            alpha = score;
            if alpha >= beta {
                if mov.is_quiet(position) {
                    memo.put_killer_move(mov, depth);
                }
                break;
            }
        }
    }

    memo.put_hash_move(zobrist_hash, best_move, depth);
    memo.put_transposition_table(zobrist_hash, depth, Some(best_move), alpha);
    (Some(best_move), alpha, count)
}

fn search_simple(
    position: &Position,
    depth: Depth,
    memo: &mut SearchMemo,
    stop_now: Option<Arc<AtomicBool>>,
) -> (Option<Move>, Score, usize) {
    alphabeta_memo(position, depth, MATE_LOWER, MATE_UPPER, memo, stop_now)
}

pub fn search_iterative_deep(
    position: &Position,
    depth: Option<Depth>,
    duration: Option<std::time::Duration>,
    stop_now: Option<Arc<AtomicBool>>,
) -> (Option<Move>, Score, usize) {
    let max_depth = depth.unwrap_or(MAX_ITERATIVE_DEPTH);
    let mut memo = SearchMemo::new(duration);

    // First guaranteed search
    let (mut mov, mut score, mut nodes) =
        search_simple(position, 1, &mut memo, stop_now.clone());
    println!("{}: {} {} {}", 1, mov.unwrap(), score, nodes);

    // Time constrained iterative deepening
    for ply in 2..=max_depth {
        let (new_mov, new_score, new_nodes) =
            search_simple(position, ply, &mut memo, stop_now.clone());

        if should_stop_search(&memo, stop_now.clone()) {
            break;
        }

        mov = new_mov;
        score = new_score;
        nodes = new_nodes;

        println!("{}: {} {} {}", ply, mov.unwrap(), score, nodes);
    }

    println!("bestmove {}", mov.unwrap());
    return (mov, score, nodes);
}

#[cfg(test)]
mod tests {
    use super::*;
    const MAX_MATE_SCORE_DIFF: Score = 300;

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
            search_iterative_deep(&position, Some(depth), None, None);
        let elapsed = now.elapsed().as_millis();
        println!(
            "[iterative] {}: {} nodes in {} ms at depth {}\n",
            fen, reg_nodes, elapsed, depth
        );

        if !test_regular {
            return;
        }

        let now = std::time::Instant::now();
        let (iter_mov, iter_score, iter_nodes) =
            search_simple(&position, depth, &mut SearchMemo::new(None), None);
        let elapsed = now.elapsed().as_millis();
        println!(
            "[regular] {}: {} nodes in {} ms at depth {}",
            fen, iter_nodes, elapsed, depth
        );

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
        test_search(
            "3k4/6R1/8/7R/8/8/8/4k3 w - - 0 1",
            5,
            "h5h8",
            Some(MATE_UPPER),
            None,
            false,
        );
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
        test_search(
            "5k2/8/6K1/5P2/8/8/8/8 w - - 0 1",
            3,
            "g6f6",
            None,
            None,
            true,
        );
    }
}
