use super::{Depth, SearchMemo};
use crate::{
    evaluation::{evaluate_game_over, evaluate_move, evaluate_position, Score},
    position::{moves::Move, Color, Position},
};

const MAX_QS_DEPTH: Depth = 10;

fn alphabeta_quiet(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    memo: &SearchMemo,
) -> (Score, usize) {
    // Check if the search should be stopped
    if memo.should_stop_search() {
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
        let a_value = evaluate_move(a, &position, false, false);
        let b_value = evaluate_move(b, &position, false, false);
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
    for mov in &moves {
        let new_position = position.make_move(mov);
        let (score, nodes) =
            alphabeta_quiet(&new_position, depth - 1, -beta, -alpha, memo);
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

pub fn alphabeta_memo(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    memo: &mut SearchMemo,
    original_depth: Depth,
) -> (Option<Move>, Score, usize) {
    // Check if the search should be stopped
    if memo.should_stop_search() {
        return (None, alpha, 1);
    }

    // Flag draw in case of threefold repetition
    let zobrist_hash = position.to_zobrist_hash();
    if memo.threefold_repetition(zobrist_hash) {
        return (None, 0, 1);
    }

    // Check for transposition table hit
    if !memo.seen_position_before(zobrist_hash) {
        if let Some(res) = memo.get_transposition_table(zobrist_hash, depth) {
            return (res.0, res.1, 1);
        }
    }

    // Cleanup tables if they get too big
    memo.cleanup_tables();

    // Enter quiescence search if depth is 0
    if depth == 0 {
        let (score, nodes) =
            alphabeta_quiet(position, MAX_QS_DEPTH, alpha, beta, memo);
        return (None, score, nodes);
    }

    // When game is over, do not search
    let mut moves = position.legal_moves(false);
    if let Some(score) =
        evaluate_game_over(position, &moves, original_depth - depth)
    {
        return (None, score as Score, 1);
    }

    // Sort moves by heuristic value + killer move + hash move
    let killer_moves = memo.get_killer_moves(depth);
    let hash_move = memo.hash_move.get(&zobrist_hash).map(|(mov, _)| mov);
    moves.sort_unstable_by(|a, b| {
        let a_value = evaluate_move(
            a,
            &position,
            SearchMemo::is_killer_move(a, killer_moves),
            SearchMemo::is_hash_move(a, hash_move),
        );
        let b_value = evaluate_move(
            b,
            &position,
            SearchMemo::is_killer_move(b, killer_moves),
            SearchMemo::is_hash_move(b, hash_move),
        );
        b_value.cmp(&a_value)
    });

    // Search moves
    let mut best_move = moves[0];
    let mut count = 0;
    for mov in &moves {
        let new_position = position.make_move(&mov);
        let new_position_hash = new_position.to_zobrist_hash();

        memo.visit_position(new_position_hash);
        let (_, score, nodes) = alphabeta_memo(
            &new_position,
            depth - 1,
            -beta,
            -alpha,
            memo,
            original_depth,
        );
        memo.leave_position(new_position_hash);

        let score = -score;
        count += nodes;

        if score > alpha {
            best_move = *mov;
            alpha = score;
            if alpha >= beta {
                if !mov.is_tactical() {
                    memo.put_killer_move(mov, depth);
                }
                break;
            }
        }
    }

    memo.put_hash_move(zobrist_hash, &best_move, depth);
    memo.put_transposition_table(zobrist_hash, depth, Some(best_move), alpha);

    (Some(best_move), alpha, count)
}
