use super::{
    constraint::SearchConstraint,
    table::{SearchTable, TTScore, TableEntry},
    Depth, MAX_DEPTH,
};
use crate::{
    evaluation::{
        moves::evaluate_move, piece_value, position::evaluate_position, Score, ValueScore,
    },
    position::{board::Piece, Color, Position},
};
use std::sync::{Arc, RwLock};

const MIN_SCORE: ValueScore = ValueScore::MIN + MAX_DEPTH + 1;
const MAX_SCORE: ValueScore = -MIN_SCORE;
const NULL_MOVE_REDUCTION: Depth = 3;
const CHECK_EXTENSION: Depth = 1;

fn quiesce(
    position: &Position,
    mut alpha: ValueScore,
    beta: ValueScore,
    constraint: &SearchConstraint,
) -> (ValueScore, usize) {
    let mut count = 1;

    let static_evaluation = evaluate_position(position) * position.side_to_move.sign();

    // Time limit reached
    if constraint.should_stop_search() {
        return (static_evaluation, count);
    }

    // Beta cutoff: position is too good
    if static_evaluation >= beta {
        return (beta, count);
    }

    // Delta pruning: sequence cannot improve the score
    if static_evaluation < alpha.saturating_sub(piece_value(Piece::Queen)) {
        return (alpha, count);
    }

    // Generate only non-quiet moves
    let mut moves = position.moves::<true>();
    moves.sort_by_cached_key(|m| -evaluate_move::<false>(position, *m));

    // Stable position reached
    if moves.is_empty() {
        return (static_evaluation, count);
    }

    // Standing pat: captures are not forced
    alpha = alpha.max(static_evaluation);

    for mov in moves.iter() {
        // Delta prune move if it cannot improve the score
        if mov.flag().is_capture() {
            let captured_piece =
                position.board.piece_color_at(mov.to()).map_or_else(|| Piece::Pawn, |p| p.0);
            if static_evaluation + piece_value(captured_piece) + 100 < alpha {
                continue;
            }
        }

        let (score, nodes) = quiesce(&position.make_move(*mov), -beta, -alpha, constraint);
        let score = -score;
        count += nodes;

        if score > alpha {
            alpha = score;

            if score >= beta {
                break;
            }
        }
    }

    (alpha, count)
}

fn pvs_recurse<const DO_NULL: bool>(
    position: &Position,
    depth: Depth,
    alpha: ValueScore,
    beta: ValueScore,
    table: Arc<RwLock<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (ValueScore, usize) {
    let mut count = 0;

    if DO_NULL {
        let (score, nodes) =
            pvs::<false>(position, depth - 1, -alpha - 1, -alpha, table.clone(), constraint);
        count += nodes;
        let score = -score;
        if score <= alpha || score >= beta {
            return (score, count);
        }
    }

    let (score, nodes) = pvs::<false>(position, depth - 1, -beta, -alpha, table, constraint);
    count += nodes;
    (-score, count)
}

fn pvs<const ROOT: bool>(
    position: &Position,
    depth: Depth,
    mut alpha: ValueScore,
    mut beta: ValueScore,
    table: Arc<RwLock<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (ValueScore, usize) {
    let mut count = 1;

    let is_twofold_repetition = constraint.is_repetition::<2>(position);
    let is_threefold_repetition = is_twofold_repetition && constraint.is_repetition::<3>(position);

    if !ROOT {
        // Detect history-related draws
        if position.halfmove_clock >= 100 || is_threefold_repetition {
            return (0, 1);
        }

        // Get known score from transposition table
        if !is_twofold_repetition {
            if let Some(tt_entry) = table.read().unwrap().get_table_score(position, depth) {
                match tt_entry {
                    TTScore::Exact(score) if score < MAX_SCORE => return (score, count),
                    TTScore::LowerBound(score) if score < MAX_SCORE => alpha = alpha.max(score),
                    TTScore::UpperBound(score) if score < MAX_SCORE => beta = beta.min(score),
                    _ => (),
                }
            }
        }

        // Time limit reached
        if constraint.should_stop_search() {
            return (alpha, count);
        }

        // Beta cutoff: position is too good
        if alpha >= beta {
            return (alpha, count);
        }
    }

    // Max depth reached; search for quiet position
    let is_check = position.is_check();
    if depth <= 0 && !is_check {
        return quiesce(position, alpha, beta, constraint);
    }

    // Null move pruning
    if !ROOT
        && !is_check
        && depth > NULL_MOVE_REDUCTION
        && position.board.piece_count(Color::White) > 0
        && position.board.piece_count(Color::Black) > 0
        && !is_twofold_repetition
    {
        let (score, nodes) = pvs::<false>(
            &position.make_null_move(),
            depth - NULL_MOVE_REDUCTION,
            -beta,
            -alpha,
            table.clone(),
            constraint,
        );

        count += nodes;
        let score = -score;

        if score >= beta {
            return (beta, count);
        }
    }

    let mut moves = position.moves::<false>();

    // Detect checkmate and stalemate
    if moves.is_empty() {
        let score = if is_check { MIN_SCORE - depth } else { 0 };
        return (score, count);
    }

    // Sort moves via MVV-LVA, psqt and table information
    let hash_move = table.read().unwrap().get_hash_move(position);
    let killer_moves = table.read().unwrap().get_killers(depth);
    moves.sort_by_cached_key(|mov| {
        if hash_move.is_some() && mov == &hash_move.unwrap() {
            return ValueScore::MIN;
        }
        if Some(*mov) == killer_moves[0] || Some(*mov) == killer_moves[1] {
            return -piece_value(Piece::Queen);
        }
        -evaluate_move::<false>(position, *mov)
    });

    let original_alpha = alpha;
    let mut best_move = moves[0];

    for (i, mov) in moves.iter().enumerate() {
        let new_position = position.make_move(*mov);

        constraint.visit_position(&new_position, mov.flag().is_reversible());
        let recurse = if i > 0 { pvs_recurse::<true> } else { pvs_recurse::<false> };
        let (score, nodes) = recurse(
            &new_position,
            if is_check { depth + CHECK_EXTENSION } else { depth },
            alpha,
            beta,
            table.clone(),
            constraint,
        );
        constraint.leave_position();

        count += nodes;

        if score > alpha {
            best_move = *mov;
            alpha = score;

            if score >= beta {
                if mov.flag().is_quiet() {
                    table.write().unwrap().put_killer_move(depth, *mov);
                }
                break;
            }
        }
    }

    if !constraint.should_stop_search() || (ROOT && depth == 1) {
        let entry = TableEntry {
            depth,
            score: if alpha <= original_alpha {
                TTScore::UpperBound(alpha)
            } else if alpha >= beta {
                TTScore::LowerBound(alpha)
            } else {
                TTScore::Exact(alpha)
            },
            best_move: Some(best_move),
        };

        table.write().unwrap().insert_entry::<ROOT>(position, entry);
    }

    (alpha, count)
}

pub fn search(
    position: &Position,
    depth: Depth,
    table: Arc<RwLock<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (Score, usize) {
    let depth = depth.min(MAX_DEPTH);

    let (score, count) = pvs::<true>(position, depth, MIN_SCORE, MAX_SCORE, table, constraint);

    if score.abs() >= MAX_SCORE {
        let moves_to_mate = (MAX_SCORE - score.abs() + depth * 2) as u8;
        (
            Score::Mate(
                if score > 0 { position.side_to_move } else { position.side_to_move.opposite() },
                (moves_to_mate + 1) / 2,
            ),
            count,
        )
    } else {
        (Score::Value(score), count)
    }
}
