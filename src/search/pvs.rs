use crate::{
    evaluate::{evaluate_position, Score, MATE_LOWER},
    position::{
        moves::{legal_moves, make_move, Move},
        Color, Position,
    },
};

use super::Searcher;

fn put_highest_score_first(position: &Position, moves: &mut Vec<Move>, start_idx: usize) {
    for i in (start_idx + 1)..moves.len() {
        let current_score = Searcher::move_heuristic_value(moves[i], position);
        let best_score = Searcher::move_heuristic_value(moves[start_idx], position);
        if current_score > best_score {
            moves.swap(i, start_idx);
        }
    }
}

pub fn pvsearch(
    searcher: &mut Searcher,
    position: &Position,
    depth: u8,
    alpha: Score,
    beta: Score,
    original_depth: u8,
    qs_depth: u8,
) -> (Option<Move>, Score, usize) {
    // Check for game over
    let mut moves = legal_moves(position, position.to_move);
    if let Some(evaluation) = Searcher::game_over_evaluation(position, &moves) {
        if evaluation == MATE_LOWER {
            return (None, MATE_LOWER + (original_depth - depth) as Score, 1); // quicker checkmate
        }
        return (None, evaluation, 1);
    }

    // Leaf: do quiet search if non quiet moves are available, otherwise evaluate statically
    let mut quiet_search = false;
    if depth == 0 {
        if qs_depth > 0 {
            if moves.iter().any(|m| !Searcher::is_quiet_move(m, position)) {
                quiet_search = true;
            }
        }

        if !quiet_search {
            let position_evaluation = evaluate_position(position);
            let score = match position.to_move {
                Color::White => position_evaluation,
                Color::Black => -position_evaluation,
            };
            return (None, score, 1);
        }
    }

    // Traverse the move tree
    let mut best_move = moves[0];
    let mut score = MATE_LOWER;
    let mut a = alpha;
    let mut node_count: usize = 0;
    for i in 0..moves.len() {
        // Get next best score move
        put_highest_score_first(position, &mut moves, i);
        let move_ = moves[i];

        // Skip quiet moves if we're in quiet search
        if quiet_search && Searcher::is_quiet_move(&move_, position) {
            continue;
        }

        // Search node
        let new_position = make_move(position, move_);
        let res = pvsearch(
            searcher,
            &new_position,
            if quiet_search || depth == 0 { 0 } else { depth - 1 },
            -beta,
            -a,
            original_depth,
            if quiet_search && qs_depth > 0 { qs_depth - 1 } else { qs_depth },
        );
        let eval = -res.1;
        node_count += res.2;

        // Update scores
        score = std::cmp::max(score, eval);
        if score > a {
            a = score;
            best_move = move_;
        }

        // Alpha-beta prune; fail soft
        if a >= beta {
            return (Some(best_move), score, node_count);
        }
    }

    (Some(best_move), score, node_count)
}
