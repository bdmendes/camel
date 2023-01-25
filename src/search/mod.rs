use crate::{
    evaluation::{
        evaluate_game_over, evaluate_move, evaluate_position, Score, MATE_LOWER, MATE_UPPER,
    },
    position::{moves::Move, zobrist::ZobristHash, Color, Position},
};
use std::collections::HashMap;

const MAX_ITERATIVE_DEPTH: i8 = 20;
const MAX_TABLE_SIZE: usize = 256_000_000;
const MAX_QSEARCH_DEPTH: i8 = 10;
const MAX_DURATION_PER_MOVE: std::time::Duration = std::time::Duration::from_secs(30);

pub type Depth = i8;

struct SearchMemo {
    pub killer_moves: HashMap<ZobristHash, ([Option<Move>; 2], Depth, Score)>,
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

    fn put_killer_move(
        &mut self,
        zobrist_hash: ZobristHash,
        mov: Move,
        depth: Depth,
        curr_score: Score,
    ) {
        let (killer_moves, killer_depth, score) =
            self.killer_moves.entry(zobrist_hash).or_insert(([None, None], 0, MATE_LOWER));
        if depth < *killer_depth {
            return;
        }
        if killer_moves[0] == None {
            killer_moves[0] = Some(mov);
        } else if killer_moves[1] == None {
            killer_moves[1] = Some(mov);
        } else if curr_score > *score {
            killer_moves[1] = killer_moves[0];
            killer_moves[0] = Some(mov);
            *score = curr_score;
        }
        *killer_depth = depth;
    }

    fn get_killer_moves(&mut self, zobrist_hash: ZobristHash) -> [Option<Move>; 2] {
        self.killer_moves.entry(zobrist_hash).or_insert(([None, None], 0, MATE_LOWER)).0
    }

    fn is_killer_move(mov: Move, killer_moves: [Option<Move>; 2]) -> bool {
        killer_moves[0] == Some(mov) || killer_moves[1] == Some(mov)
    }

    fn put_hash_move(&mut self, zobrist_hash: ZobristHash, mov: Move, depth: Depth) {
        let (hash_mov, hash_depth) = self.hash_move.entry(zobrist_hash).or_insert((mov, 0));
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
        let (transp_mov, transp_score, transp_depth) =
            self.transposition_table.entry(zobrist_hash).or_insert((mov, score, 0));
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
        if let Some((mov, score, transp_depth)) = self.transposition_table.get(&zobrist_hash) {
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

fn alphabeta(
    position: &Position,
    depth: Depth,
    alpha: Score,
    beta: Score,
    memo: &mut SearchMemo,
    qs_depth: Depth,
) -> (Option<Move>, Score, usize) {
    // Check if time is over
    if memo.initial_instant.elapsed() > memo.duration {
        return (None, alpha, 1);
    }

    // Cleanup tables if they get too big
    memo.cleanup_tables();

    // Check for transposition table
    let zobrist_hash = position.to_zobrist_hash();
    if let Some(res) = memo.get_transposition_table(zobrist_hash, depth) {
        return (res.0, res.1, 1);
    }

    // Check for game over
    let mut moves = position.legal_moves();
    if let Some(score) = evaluate_game_over(position, &moves) {
        return (None, score + position.half_move_number as Score, 1);
    }

    // Check for maximum depth
    let mut quiet_search = false;
    if depth <= 0 {
        if qs_depth > 0 {
            quiet_search = moves.iter().any(|mov| !mov.is_quiet(position));
        }

        if !quiet_search {
            let color_cof = match position.to_move {
                Color::White => 1,
                Color::Black => -1,
            };
            return (None, color_cof * evaluate_position(position), 1);
        }
    }

    // Sort moves by heuristic value + killer move + hash move
    let killer_moves = memo.get_killer_moves(zobrist_hash);
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
    let mut best_score = alpha;
    let mut count = 0;
    for mov in &moves {
        if quiet_search && mov.is_quiet(position) {
            continue;
        }

        let new_position = position.make_move(*mov);
        let (_, score, nodes) = alphabeta(
            &new_position,
            depth - 1,
            -beta,
            -best_score,
            memo,
            if quiet_search { qs_depth - 1 } else { qs_depth },
        );
        let score = -score;
        count += nodes;

        if score > best_score {
            best_move = *mov;
            best_score = score;
            if best_score >= beta {
                if mov.is_quiet(position) {
                    memo.put_killer_move(zobrist_hash, *mov, depth, best_score);
                }
                break;
            }
        }
    }

    memo.put_hash_move(zobrist_hash, best_move, depth);
    memo.put_transposition_table(zobrist_hash, depth, Some(best_move), best_score);
    (Some(best_move), best_score, count)
}

pub fn search(position: &Position, depth: Depth) -> (Option<Move>, Score, usize) {
    alphabeta(
        position,
        depth,
        MATE_LOWER,
        MATE_UPPER,
        &mut SearchMemo::new(None),
        MAX_QSEARCH_DEPTH,
    )
}

pub fn search_iterative_deep(
    position: &Position,
    depth: Option<Depth>,
    duration: Option<std::time::Duration>,
) -> (Option<Move>, Score, usize) {
    let max_depth = depth.unwrap_or(MAX_ITERATIVE_DEPTH);
    let mut memo = SearchMemo::new(duration);

    for ply in 1..=max_depth {
        let (mov, score, nodes) =
            alphabeta(position, ply, MATE_LOWER, MATE_UPPER, &mut memo, MAX_QSEARCH_DEPTH);

        if ply > 1 && memo.initial_instant.elapsed() > memo.duration {
            return (mov, score, nodes);
        }

        println!("{}: {} {} {}", ply, mov.unwrap(), score, nodes);

        if ply == max_depth {
            return (mov, score, nodes);
        }
    }

    unreachable!()
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
    ) {
        let position = Position::from_fen(fen).unwrap();

        let now = std::time::Instant::now();
        let (mov, score, nodes) = search(&position, depth);
        let elapsed = now.elapsed().as_millis();
        println!("[regular] {}: {} nodes in {} ms at depth {}", fen, nodes, elapsed, depth);

        assert_eq!(mov.unwrap().to_string(), expected_move);
        if let Some(expected_lower_score) = expected_lower_score {
            assert!(score >= expected_lower_score);
        }
        if let Some(expected_upper_score) = expected_upper_score {
            assert!(score <= expected_upper_score);
        }

        let now = std::time::Instant::now();
        let (mov, score, nodes) = search_iterative_deep(&position, Some(depth), None);
        let elapsed = now.elapsed().as_millis();
        println!("[iterative] {}: {} nodes in {} ms at depth {}", fen, nodes, elapsed, depth);

        assert_eq!(mov.unwrap().to_string(), expected_move);
        if let Some(expected_lower_score) = expected_lower_score {
            assert!(score >= expected_lower_score);
        }
        if let Some(expected_upper_score) = expected_upper_score {
            assert!(score <= expected_upper_score);
        }
    }

    #[test]
    fn search_mate_in_1() {
        test_search(
            "3k4/6R1/8/7R/8/8/8/4k3 w - - 0 1",
            5,
            "h5h8",
            Some(MATE_UPPER - MAX_MATE_SCORE_DIFF),
            None,
        );
    }

    #[test]
    fn search_forced_capture() {
        test_search(
            "r2N2k1/p1R3pp/8/1pP5/1b2p1bP/4P3/4BPP1/3K3R b - - 0 24",
            5,
            "a8d8",
            None,
            None,
        );
    }
}
