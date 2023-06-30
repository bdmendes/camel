use crate::{
    evaluation::{
        moves::evaluate_move, piece_value, position::evaluate_position, Score, ValueScore,
    },
    position::{board::Piece, Color, Position},
};

use super::{
    table::{SearchTable, TTEntry, TTScore},
    Depth,
};

const MIN_SCORE: ValueScore = ValueScore::MIN + 1;
const NULL_MOVE_REDUCTION: Depth = 3;
const CHECK_EXTENSION: Depth = 1;

fn quiesce(
    position: &Position,
    mut alpha: ValueScore,
    beta: ValueScore,
    table: &SearchTable,
) -> (ValueScore, usize) {
    let static_evaluation = evaluate_position(position) * position.side_to_move.sign();

    // Time limit reached
    if table.should_stop_search() {
        return (static_evaluation, 1);
    }

    // Beta cutoff: position is too good
    if static_evaluation >= beta {
        return (beta, 1);
    }

    // Delta pruning: sequence cannot improve the score
    if static_evaluation < alpha.saturating_sub(piece_value(Piece::Queen)) {
        return (alpha, 1);
    }

    // Generate only non-quiet moves
    let mut moves = position.moves::<true>();
    moves.sort_by_cached_key(|m| -evaluate_move::<false>(position, *m));

    // Stable position reached
    if moves.is_empty() {
        return (static_evaluation, 1);
    }

    // Standing pat: captures are not forced
    alpha = alpha.max(static_evaluation);

    let mut count = 0;
    for mov in moves.iter() {
        let (score, nodes) = quiesce(&position.make_move(*mov), -beta, -alpha, table);
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

fn pvs_recurse(
    position: &Position,
    depth: Depth,
    alpha: ValueScore,
    beta: ValueScore,
    table: &mut SearchTable,
    original_depth: Depth,
    do_null: bool,
) -> (ValueScore, usize) {
    let mut count = 0;

    if do_null {
        let (score, nodes) = pvs(position, depth - 1, -alpha - 1, -alpha, table, original_depth);
        count += nodes;
        let score = -score;
        if score <= alpha || score >= beta {
            return (score, count);
        }
    }

    let (score, nodes) = pvs(position, depth - 1, -beta, -alpha, table, original_depth);
    count += nodes;
    (-score, count)
}

fn pvs(
    position: &Position,
    depth: Depth,
    mut alpha: ValueScore,
    mut beta: ValueScore,
    table: &mut SearchTable,
    original_depth: Depth,
) -> (ValueScore, usize) {
    // Get known score from transposition table
    if let Some(tt_entry) = table.get_table_score(position, depth) {
        match tt_entry {
            TTScore::Exact(score) => return (score, 1),
            TTScore::LowerBound(score) => alpha = alpha.max(score),
            TTScore::UpperBound(score) => beta = beta.min(score),
        }
    }

    // Time limit reached
    if table.should_stop_search() {
        return (alpha, 1);
    }

    // Beta cutoff: position is too good
    if alpha >= beta {
        return (alpha, 1);
    }

    // Max depth reached; search for quiet position
    let is_check = position.is_check();
    if depth <= 0 && !is_check {
        return quiesce(position, alpha, beta, table);
    }

    let mut moves = position.moves::<false>();

    // Detect checkmate, stalemate and threefold repetition
    if moves.is_empty() {
        let score = if is_check { MIN_SCORE + original_depth - depth } else { 0 };
        return (score, 1);
    } else if position.halfmove_clock >= 50 || table.is_threefold_repetition(position) {
        return (0, 1);
    }

    let mut count = 0;

    // Null move pruning
    if depth != original_depth
        && !is_check
        && depth > NULL_MOVE_REDUCTION
        && position.board.piece_count(Color::White) > 0
        && position.board.piece_count(Color::Black) > 0
    {
        let (score, nodes) = pvs(
            &position.make_null_move(),
            depth - NULL_MOVE_REDUCTION,
            -beta,
            -alpha,
            table,
            original_depth,
        );

        count += nodes;
        let score = -score;

        if score >= beta {
            return (beta, count);
        }
    }

    // Sort moves via MVV-LVA, psqt and table information
    let hash_move = table.get_hash_move(position);
    let killer_moves = table.get_killers(depth);
    moves.sort_by_cached_key(move |mov| {
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

        table.visit_position(&new_position);
        let (score, nodes) = pvs_recurse(
            &new_position,
            if is_check { depth + CHECK_EXTENSION } else { depth },
            alpha,
            beta,
            table,
            original_depth,
            i > 0,
        );
        table.leave_position();

        count += nodes;

        if score > alpha {
            best_move = *mov;
            alpha = score;

            if score >= beta {
                if mov.flag().is_quiet() {
                    table.put_killer_move(depth, *mov);
                }
                break;
            }
        }
    }

    if !table.should_stop_search() {
        table.insert_entry(
            position,
            TTEntry {
                depth,
                score: if alpha <= original_alpha {
                    TTScore::UpperBound(alpha)
                } else if alpha >= beta {
                    TTScore::LowerBound(alpha)
                } else {
                    TTScore::Exact(alpha)
                },
                best_move: Some(best_move),
            },
        );
    }

    (alpha, count)
}

pub fn search(position: &Position, depth: Depth, table: &mut SearchTable) -> (Score, usize) {
    let (score, count) = pvs(position, depth, ValueScore::MIN + 1, ValueScore::MAX, table, depth);
    if score.abs() >= ValueScore::MAX - depth - 1 {
        (
            Score::Mate(
                if score > 0 { position.side_to_move } else { position.side_to_move.opposite() },
                ((ValueScore::MAX - score.abs()) / 2) as u8 + 1,
            ),
            count,
        )
    } else {
        (Score::Value(score), count)
    }
}
