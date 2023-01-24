use crate::{
    evaluate::{evaluate_position, Evaluation, Score},
    position::{
        moves::{legal_moves, make_move, Move},
        Color, Position,
    },
};

use super::{Bound, BoundEvaluation, Searcher};

pub fn pvsearch(
    searcher: &mut Searcher,
    position: &Position,
    depth: u8,
    alpha: Evaluation,
    beta: Evaluation,
    original_depth: u8,
    quiet_search_depth: u8,
) -> (Option<Vec<(Move, BoundEvaluation)>>, BoundEvaluation) {
    // Check table
    let zobrist_hash = position.to_zobrist_hash();
    if let Some((bound_evaluation, found_depth)) = searcher.position_value_table.get(&zobrist_hash)
    {
        if *found_depth >= depth {
            //println!("Found position in table");
            return (None, *bound_evaluation);
        }
    }

    let mut moves = legal_moves(position, position.to_move);

    // Check for game over
    if let Some(evaluation) = Searcher::game_over_evaluation(position, &moves) {
        let bound = BoundEvaluation::new(evaluation, Bound::Exact);
        return (None, bound);
    }

    let mut quiet_search = false;
    if depth == 0 {
        // Switch to quiet search if there are non-quiet moves
        if quiet_search_depth > 0 {
            if moves.iter().any(|m| !Searcher::is_quiet_move(m, position)) {
                quiet_search = true;
            }
        }

        // No depth left at quiet position, evaluate at leaf node
        if !quiet_search {
            let position_evaluation = evaluate_position(position);
            let score = match position.to_move {
                Color::White => position_evaluation,
                Color::Black => -position_evaluation,
            };
            let bound = BoundEvaluation::new(score, Bound::Exact);
            return (None, bound);
        }
    }

    // Order moves by heuristic/table value
    moves.sort_by(|m1, m2| {
        let m2_table_value = searcher.move_value_table.get(&(m2.clone(), zobrist_hash));
        let m1_table_value = searcher.move_value_table.get(&(m1.clone(), zobrist_hash));
        m2_table_value
            .unwrap_or(&Searcher::move_heuristic_value(m2, position))
            .cmp(m1_table_value.unwrap_or(&Searcher::move_heuristic_value(m1, position)))
    });

    if depth == original_depth {
        println!("First move: {}", moves[0]);
    }

    // Traverse the move tree
    let mut eval_moves = Vec::new();
    let mut best_score = f32::MIN;
    let mut new_alpha = alpha;
    let mut before_principal = true;
    for move_ in moves {
        // Skip quiet moves if we're in quiet search
        if quiet_search && Searcher::is_quiet_move(&move_, position) {
            continue;
        }

        // Search node
        let new_position = make_move(position, &move_);
        let eval = pvsearch(
            searcher,
            &new_position,
            if quiet_search || depth == 0 {
                0
            } else {
                depth - 1
            },
            if before_principal || quiet_search {
                -beta
            } else {
                -new_alpha - 1.0
            }, // TODO: debug principal variation search
            -new_alpha,
            original_depth,
            if quiet_search && quiet_search_depth > 0 {
                quiet_search_depth - 1
            } else {
                quiet_search_depth
            },
        )
        .1;
        let score = -eval.evaluation;

        // If we're at the original depth, store the move
        if depth == original_depth {
            eval_moves.push((move_.clone(), BoundEvaluation::new(score, eval.bound)));
        }

        // Update best score
        if score > best_score {
            best_score = score;
        }
        if score > new_alpha {
            new_alpha = score;
        }

        // Update move value table
        searcher
            .move_value_table
            .insert((move_, zobrist_hash), (score * 100.0) as Score);

        // Alpha-beta pruning; fail soft
        if new_alpha >= beta {
            return (None, BoundEvaluation::new(new_alpha, Bound::Upper));
        }

        before_principal = false;
    }

    // Sort moves by score
    eval_moves.sort_by(|a, b| b.1.evaluation.partial_cmp(&a.1.evaluation).unwrap());

    // Update position value table
    let bound = if best_score <= alpha {
        Bound::Upper
    } else if best_score >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };
    searcher.position_value_table.insert(
        zobrist_hash,
        (BoundEvaluation::new(best_score, bound), depth),
    );

    (
        Some(eval_moves),
        BoundEvaluation::new(new_alpha, Bound::Exact),
    )
}
