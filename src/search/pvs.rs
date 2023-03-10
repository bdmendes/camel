use super::{Depth, Node, SearchMemo};
use crate::{
    evaluation::{
        moves::evaluate_move,
        piece_value,
        position::{evaluate_game_over, evaluate_position},
        Score,
    },
    position::{moves::Move, Color, Piece, Position},
};

const NULL_MOVE_REDUCTION: Depth = 3;
const CHECK_EXTENSION: Depth = 1;
const MAX_QS_DEPTH: Depth = 10;
const OPENING_MOVE_THRESHOLD: u16 = 5;

fn quiesce_search(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    memo: &mut SearchMemo,
    opening_entropy: bool,
) -> (Score, usize) {
    // Check if the search should be stopped
    if memo.should_stop_search() {
        return (alpha, 1);
    }

    // Calculate static evaluation and return if quiescence search depth is reached
    let static_evaluation = evaluate_position(position, opening_entropy, true);
    if depth <= 0 {
        return (static_evaluation, 1);
    }

    // Alpha-beta prune based on static evaluation
    if static_evaluation >= beta {
        return (beta, 1);
    }

    // Delta pruning: prune if this capture sequence cannot improve the score
    if static_evaluation < alpha - piece_value(Piece::WQ) {
        return (alpha, 1);
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
            quiesce_search(&new_position, depth - 1, -beta, -alpha, memo, opening_entropy);
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

fn pvs_recurse(
    position: &Position,
    depth: Depth,
    alpha: Score,
    beta: Score,
    memo: &mut SearchMemo,
    original_depth: Depth,
    zero_window_search: bool,
) -> (Score, usize) {
    let mut count = 0;

    if zero_window_search {
        let (_, score, nodes) = pvs(position, depth - 1, -alpha - 1, -alpha, memo, original_depth);
        count += nodes;
        let score = -score;
        if score <= alpha || score >= beta {
            return (score, count);
        }
    }

    let (_, score, nodes) = pvs(position, depth - 1, -beta, -alpha, memo, original_depth);
    count += nodes;
    let score = -score;
    return (score, count);
}

pub fn pvs(
    position: &Position,
    depth: Depth,
    mut alpha: Score,
    mut beta: Score,
    memo: &mut SearchMemo,
    original_depth: Depth,
) -> (Option<Move>, Score, usize) {
    // Check if the search should be stopped
    if memo.should_stop_search() {
        return (None, alpha, 1);
    }

    // Check for transposition table hit
    let zobrist_hash = position.zobrist_hash();
    if let Some(tt_entry) = memo.get_transposition_table(zobrist_hash, depth) {
        match tt_entry.node {
            Node::PVNode => return (None, tt_entry.score, 1),
            Node::CutNode => {
                if tt_entry.score > alpha {
                    alpha = tt_entry.score;
                }
            }
            Node::AllNode => {
                if tt_entry.score < beta {
                    beta = tt_entry.score;
                }
            }
        }

        if alpha >= beta {
            return (None, tt_entry.score, 1);
        }
    }

    // Enter quiescence search if depth is 0 and not in check
    let is_check = position.is_check();
    if depth <= 0 && !is_check {
        let (score, nodes) = quiesce_search(
            position,
            MAX_QS_DEPTH,
            alpha,
            beta,
            memo,
            (position.full_move_number * 2 - original_depth as u16) < OPENING_MOVE_THRESHOLD,
        );
        return (None, score, nodes);
    }

    // When game is over, do not search
    let mut moves = position.legal_moves(false);
    if let Some(score) =
        evaluate_game_over(position, &moves, original_depth - depth, Some(&memo.branch_history))
    {
        return (None, score as Score, 1);
    }

    // Null move pruning when not in check and zugzwang is not possible
    if depth != original_depth
        && depth > NULL_MOVE_REDUCTION
        && !is_check
        && position.piece_count(Some(Color::White), None) > 0
        && position.piece_count(Some(Color::Black), None) > 0
    {
        let new_position = position.make_null_move();
        let (_, score, nodes) =
            pvs(&new_position, depth - NULL_MOVE_REDUCTION, -beta, -alpha, memo, original_depth);

        let score = -score;
        if score >= beta {
            return (None, beta, nodes);
        }
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
    let original_alpha = alpha;
    let mut best_move = moves[0];
    let mut count = 0;
    for mov in &moves {
        let new_position = position.make_move(&mov);

        memo.visit_position(new_position.zobrist_hash());
        let (score, nodes) = pvs_recurse(
            &new_position,
            if is_check { depth + CHECK_EXTENSION } else { depth },
            alpha,
            beta,
            memo,
            original_depth,
            count > 0,
        );
        memo.leave_position();

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
    memo.put_transposition_table(
        zobrist_hash,
        depth,
        Some(best_move),
        alpha,
        if alpha >= beta {
            Node::CutNode
        } else if alpha <= original_alpha {
            Node::AllNode
        } else {
            Node::PVNode
        },
    );

    (Some(best_move), alpha, count)
}
