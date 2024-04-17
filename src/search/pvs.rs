use super::{
    constraint::SearchConstraint,
    history::BranchHistory,
    movepick::MovePicker,
    table::{SearchTable, TableScore},
    Depth, MAX_DEPTH,
};
use crate::{
    evaluation::{position::MAX_POSITIONAL_GAIN, Evaluable, Score, ValueScore},
    position::{board::Piece, Color, Position},
};
use std::{cell::OnceCell, sync::Arc};

const MATE_SCORE: ValueScore = ValueScore::MIN + MAX_DEPTH as ValueScore + 1;
const NULL_MOVE_DEPTH_REDUCTION: Depth = 3;
const WINDOW_SIZE: ValueScore = 100;

fn quiesce(
    position: &Position,
    mut alpha: ValueScore,
    beta: ValueScore,
    constraint: &SearchConstraint,
) -> (ValueScore, usize) {
    // Time limit reached
    if constraint.should_stop_search() {
        return (alpha, 1);
    }

    // If we are in check, the position is certainly not quiet,
    // so we must search all check evasions. Otherwise, search only captures
    let is_check = position.is_check();
    let static_evaluation = if is_check {
        alpha
    } else {
        let static_evaluation = position.value() * position.side_to_move.sign();

        // Standing pat: captures are not forced
        alpha = alpha.max(static_evaluation);

        // Beta cutoff: position is too good
        if static_evaluation >= beta {
            return (beta, 1);
        }

        // Delta pruning: sequence cannot improve the score
        if static_evaluation < alpha.saturating_sub(Piece::Queen.value()) {
            return (alpha, 1);
        }

        static_evaluation
    };

    let mut picker = MovePicker::<true>::new(position, is_check).peekable();

    // Stable position reached
    if picker.peek().is_none() {
        let score = if is_check { MATE_SCORE } else { static_evaluation };
        return (score, 1);
    }

    let mut count = 1;

    for (mov, _) in picker {
        // Delta prune move if it cannot improve the score
        if !is_check && mov.flag().is_capture() {
            let captured_piece =
                position.board.piece_color_at(mov.to()).map_or_else(|| Piece::Pawn, |p| p.0);
            if static_evaluation + captured_piece.value() + MAX_POSITIONAL_GAIN < alpha {
                continue;
            }
        }

        let (score, nodes) = quiesce(&position.make_move(mov), -beta, -alpha, constraint);
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

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn pvs_recurse<const MAIN_THREAD: bool>(
    position: &mut Position,
    current_depth: Depth,
    alpha: ValueScore,
    beta: ValueScore,
    table: Arc<SearchTable>,
    constraint: &SearchConstraint,
    history: &mut BranchHistory,
    do_zero_window: bool,
    reduction: Depth,
    extension: Depth,
) -> (ValueScore, usize) {
    let mut count = 0;

    if do_zero_window {
        // We expect this tree to not raise alpha, so we search with tight bounds.
        let (score, nodes) = pvs::<false, MAIN_THREAD, true>(
            position,
            current_depth.saturating_add(extension).saturating_sub(reduction + 1),
            -alpha - 1,
            -alpha,
            table.clone(),
            constraint,
            history,
        );
        count += nodes;
        let score = -score;
        if score <= alpha || score >= beta {
            // We did not exceed alpha, so our fast search is ok.
            return (score, count);
        }
    }

    // We found a better move, so we must search with full window to confirm.
    // We also eliminate the reduction to avoid missing deep lines.
    let (score, nodes) = pvs::<false, MAIN_THREAD, true>(
        position,
        current_depth.saturating_add(extension).saturating_sub(1),
        -beta,
        -alpha,
        table,
        constraint,
        history,
    );
    count += nodes;
    (-score, count)
}

fn may_be_zugzwang(position: &Position) -> bool {
    let pawns_bb = position.board.pieces_bb(Piece::Pawn);
    let kings_bb = position.board.pieces_bb(Piece::King);

    let white_pieces_bb = position.board.occupancy_bb(Color::White) & !pawns_bb & !kings_bb;
    let black_pieces_bb = position.board.occupancy_bb(Color::Black) & !pawns_bb & !kings_bb;

    white_pieces_bb.is_empty() || black_pieces_bb.is_empty()
}

fn pvs<const ROOT: bool, const MAIN_THREAD: bool, const ALLOW_NMR: bool>(
    position: &mut Position,
    mut depth: Depth,
    mut alpha: ValueScore,
    mut beta: ValueScore,
    table: Arc<SearchTable>,
    constraint: &SearchConstraint,
    history: &mut BranchHistory,
) -> (ValueScore, usize) {
    let repeated_times = history.repeated(position);
    let twofold_repetition = repeated_times >= 2;
    let threefold_repetition = repeated_times >= 3;

    // Detect history-related draws
    if position.halfmove_clock >= 100 || threefold_repetition {
        return (0, 1);
    }

    // Get known score from transposition table
    if !twofold_repetition {
        if let Some(tt_entry) = table.get_table_score(position, depth) {
            match tt_entry {
                TableScore::Exact(score) => return (score, 1),
                TableScore::LowerBound(score) => alpha = alpha.max(score),
                TableScore::UpperBound(score) => beta = beta.min(score),
            }

            // Beta cutoff: position is too good
            if alpha >= beta {
                return (alpha, 1);
            }
        }
    }

    // Time limit reached
    if constraint.should_stop_search() {
        return (alpha, 1);
    }

    // Max depth reached; search for quiet position
    if depth == 0 {
        return quiesce(position, alpha, beta, constraint);
    }

    // We count each move on the board as 1 node.
    let mut count = 1;

    // Position and node type considerations.
    let is_check = position.is_check();
    let may_be_zug = may_be_zugzwang(position);

    // Null move pruning: if we "pass" our turn and still get a beta cutoff,
    // this position is far too good to be true.
    // We must not allow repeated null moves, otherwise we'll end up
    // in the same position.
    if !ROOT
        && ALLOW_NMR
        && !is_check
        && !twofold_repetition
        && depth > NULL_MOVE_DEPTH_REDUCTION
        && !may_be_zug
    {
        position.side_to_move = position.side_to_move.opposite();
        let (score, nodes) = pvs::<false, MAIN_THREAD, false>(
            position,
            depth - NULL_MOVE_DEPTH_REDUCTION,
            -beta,
            -alpha,
            table.clone(),
            constraint,
            history,
        );
        position.side_to_move = position.side_to_move.opposite();

        count += nodes;
        let score = -score;

        if score >= beta {
            return (beta, count);
        }
    }

    // Prepare move generation and sorting. This is lazy and works in stages.
    let mut picker =
        MovePicker::<false>::new(position, table.clone(), depth, ROOT && !MAIN_THREAD).peekable();

    // Detect checkmate and stalemate
    if picker.peek().is_none() {
        let score = if is_check { MATE_SCORE - depth as ValueScore } else { 0 };
        return (score, count);
    }

    // Check extension: we are interested in exploring the outcome of this properly.
    if is_check {
        depth = depth.saturating_add(1).min(MAX_DEPTH);
    }

    // The static evaluation is useful for pruning techniques,
    // but might not be needed.
    let static_evaluation = OnceCell::new();

    // We need to keep track of the original alpha and best moves, to store
    // the correct node type and move in the hash table later.
    let original_alpha = alpha;
    let mut best_move = picker.peek().map(|(mov, _)| *mov).unwrap();

    for (i, (mov, _)) in picker.enumerate() {
        // Extended futility pruning: discard moves without potential
        if depth <= 2 && i > 0 && !may_be_zug {
            let move_potential = MAX_POSITIONAL_GAIN * depth as ValueScore
                + mov
                    .flag()
                    .is_capture()
                    .then(|| position.board.piece_at(mov.to()).unwrap_or(Piece::Pawn).value())
                    .unwrap_or(0);
            if static_evaluation.get_or_init(|| position.value() * position.side_to_move.sign())
                + move_potential
                < alpha
            {
                continue;
            }
        }

        // Late move reduction: we assume our move ordering is good, and are less interested in
        // expected non-PV nodes.
        let late_move_reduction =
            if depth >= 3 && !is_check && mov.flag().is_quiet() && i > 0 { depth / 3 } else { 0 };

        let mut new_position = position.make_move(mov);

        history.visit_position(&new_position, mov.flag().is_reversible());
        let (score, nodes) = pvs_recurse::<MAIN_THREAD>(
            &mut new_position,
            depth,
            alpha,
            beta,
            table.clone(),
            constraint,
            history,
            i > 0,
            late_move_reduction,
            0,
        );
        history.leave_position();

        count += nodes;

        if score > alpha {
            // We found a new best move.
            best_move = mov;
            alpha = score;

            if score >= beta {
                if MAIN_THREAD && mov.flag().is_quiet() {
                    // Killer moves are prioritized in move ordering.
                    // It assumes that similar "refutation" moves at siblings will be useful.
                    table.put_killer_move(depth, mov);
                }

                // This position is now far too good to be true.
                // We can safely skip remaining moves.
                break;
            }
        }
    }

    if !constraint.should_stop_search() {
        table.insert_entry(
            position,
            if alpha <= original_alpha {
                TableScore::UpperBound(alpha)
            } else if alpha >= beta {
                TableScore::LowerBound(alpha)
            } else {
                TableScore::Exact(alpha)
            },
            best_move,
            depth,
            ROOT && MAIN_THREAD,
        );
    }

    (alpha, count)
}

pub fn pvs_aspiration<const MAIN_THREAD: bool>(
    position: &Position,
    guess: ValueScore,
    depth: Depth,
    table: Arc<SearchTable>,
    constraint: &SearchConstraint,
) -> Option<(Score, usize)> {
    let depth = depth.min(MAX_DEPTH);
    let mut position = *position;
    let mut all_count = 0;
    let mut lower_bound = guess - WINDOW_SIZE;
    let mut upper_bound = guess + WINDOW_SIZE;

    if MAIN_THREAD {
        // We must update the search id so that new table entries may be pushed.
        table.prepare_for_new_search(&position);
    }

    for cof in 1.. {
        let (score, count) = pvs::<true, MAIN_THREAD, true>(
            &mut position,
            depth,
            lower_bound,
            upper_bound,
            table.clone(),
            constraint,
            &mut BranchHistory(constraint.game_history.clone()),
        );
        all_count += count;

        if !constraint.should_stop_search() {
            // Search failed low; increase lower bound and try again
            if score <= lower_bound {
                lower_bound = std::cmp::max(
                    ValueScore::MIN + 1,
                    lower_bound.saturating_sub(WINDOW_SIZE.saturating_mul(cof)),
                );
                continue;
            }

            // Search failed high; increase upper bound and try again
            if score >= upper_bound {
                upper_bound = upper_bound.saturating_add(WINDOW_SIZE.saturating_mul(cof));
                continue;
            }

            // Our score is valid, so other threads can stop gracefully.
            if MAIN_THREAD {
                constraint.signal_root_finished();
            }

            return Some(if score.abs() >= MATE_SCORE.abs() {
                let pv = table.get_pv(&position, depth);
                let plys_to_mate = pv.len() as u8;
                (
                    Score::Mate(
                        if score > 0 {
                            position.side_to_move
                        } else {
                            position.side_to_move.opposite()
                        },
                        (plys_to_mate + 1) / 2,
                    ),
                    all_count,
                )
            } else {
                (Score::Value(score), all_count)
            });
        }

        // Search stopped as an outside order, so this is not a valid result.
        return None;
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::fen::FromFen, search::table::DEFAULT_TABLE_SIZE_MB};

    fn expect_pvs_aspiration(
        fen: &str,
        depth: Depth,
        expected_moves: Vec<&str>,
        expected_score: Option<Score>,
    ) {
        let position = Position::from_fen(fen).unwrap();
        let table = Arc::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB));
        let constraint = SearchConstraint::default();

        let score =
            pvs_aspiration::<true>(&position, 0, depth, table.clone(), &constraint).unwrap().0;
        let pv = table.get_pv(&position, depth);

        assert!(pv.len() >= expected_moves.len());

        for (mov, i) in pv.iter().map(|m| m.to_string()).enumerate() {
            if mov >= expected_moves.len() {
                break;
            }
            assert_eq!(i, expected_moves[mov]);
        }

        if let Some(expected_score) = expected_score {
            assert_eq!(score, expected_score);
        }
    }

    #[test]
    fn mate_us_1() {
        expect_pvs_aspiration(
            "rnb1r2k/pR3Qpp/2p5/4N3/3P3P/2q5/P2p1PP1/5K1R w - - 1 20",
            2,
            vec!["f7e8"],
            Some(Score::Mate(Color::White, 1)),
        );
    }

    #[test]
    fn mate_them_2() {
        expect_pvs_aspiration(
            "rnb1r1k1/pR3ppp/2p5/4N3/3P1Q1P/3p4/P4PP1/q4K1R w - - 3 19",
            6,
            vec!["b7b1", "a1b1", "f4c1", "b1c1"],
            Some(Score::Mate(Color::Black, 2)),
        );
    }

    #[test]
    fn mate_us_3() {
        expect_pvs_aspiration(
            "rnb1r1k1/pR3ppp/2p5/4N3/3P1Q1P/2qp4/P4PP1/5K1R b - - 2 18",
            7,
            vec!["c3a1", "b7b1", "a1b1", "f4c1", "b1c1"],
            Some(Score::Mate(Color::Black, 3)),
        );
    }
}
