use super::{
    constraint::SearchConstraint,
    movepick::MovePicker,
    table::{SearchTable, TableEntry, TableScore},
    Depth, MAX_DEPTH,
};
use crate::{
    evaluation::{position::MAX_POSITIONAL_GAIN, Evaluable, Score, ValueScore},
    position::{board::Piece, Color, Position},
};
use std::sync::{Arc, Mutex};

const MATE_SCORE: ValueScore = ValueScore::MIN + MAX_DEPTH as ValueScore + 1;

pub fn quiesce(
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

fn pvs_recurse(
    position: &mut Position,
    depth: Depth,
    alpha: ValueScore,
    beta: ValueScore,
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
    do_zero_window: bool,
) -> (ValueScore, usize) {
    let mut count = 0;

    if do_zero_window {
        let (score, nodes) =
            pvs::<false>(position, depth, -alpha - 1, -alpha, table.clone(), constraint);
        count += nodes;
        let score = -score;
        if score <= alpha || score >= beta {
            return (score, count);
        }
    }

    let (score, nodes) = pvs::<false>(position, depth, -beta, -alpha, table, constraint);
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

fn pvs<const ROOT: bool>(
    position: &mut Position,
    depth: Depth,
    mut alpha: ValueScore,
    mut beta: ValueScore,
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (ValueScore, usize) {
    if ROOT {
        table.lock().unwrap().prepare_for_new_search(position.fullmove_number);
    }

    let repeated_times = constraint.repeated(position);
    let twofold_repetition = repeated_times >= 2;
    let threefold_repetition = repeated_times >= 3;

    if !ROOT {
        // Detect history-related draws
        if position.halfmove_clock >= 100 || threefold_repetition {
            return (0, 1);
        }

        // Get known score from transposition table
        if !twofold_repetition {
            if let Some(tt_entry) = table.lock().unwrap().get_table_score(position, depth) {
                match tt_entry {
                    TableScore::Exact(score) => return (score, 1),
                    TableScore::LowerBound(score) => alpha = alpha.max(score),
                    TableScore::UpperBound(score) => beta = beta.min(score),
                }
            }
        }

        // Beta cutoff: position is too good
        if alpha >= beta {
            return (alpha, 1);
        }

        // Time limit reached
        if constraint.should_stop_search() {
            return (alpha, 1);
        }
    }

    // Max depth reached; search for quiet position
    if depth == 0 {
        return quiesce(position, alpha, beta, constraint);
    }

    let mut count = 1;

    let is_check = position.is_check();

    // Null move pruning
    if !ROOT && !is_check && !twofold_repetition && depth > 3 && !may_be_zugzwang(position) {
        position.side_to_move = position.side_to_move.opposite();
        let (score, nodes) =
            pvs::<false>(position, depth - 3, -beta, -alpha, table.clone(), constraint);
        position.side_to_move = position.side_to_move.opposite();

        count += nodes;
        let score = -score;

        if score >= beta {
            return (beta, count);
        }
    }

    let mut picker = MovePicker::<false>::new(position, table.clone(), depth).peekable();

    // Detect checkmate and stalemate
    if picker.peek().is_none() {
        let score = if is_check { MATE_SCORE - depth as ValueScore } else { 0 };
        return (score, count);
    }

    let original_alpha = alpha;
    let mut best_move = picker.peek().map(|(mov, _)| *mov).unwrap();

    for (i, (mov, _)) in picker.enumerate() {
        let mut new_position = position.make_move(mov);
        let new_depth = depth
            .saturating_sub(1)
            .saturating_sub(if depth > 2 && !is_check && mov.flag().is_quiet() && i > 0 {
                1
            } else {
                0
            })
            .saturating_add(if is_check { 1 } else { 0 });

        constraint.visit_position(&new_position, mov.flag().is_reversible());
        let (score, nodes) = pvs_recurse(
            &mut new_position,
            new_depth,
            alpha,
            beta,
            table.clone(),
            constraint,
            i > 0,
        );
        constraint.leave_position();

        count += nodes;

        if score > alpha {
            best_move = mov;
            alpha = score;

            if score >= beta {
                if mov.flag().is_quiet() {
                    table.lock().unwrap().put_killer_move(depth, mov);
                }
                break;
            }
        }
    }

    if !constraint.should_stop_search() {
        let entry = TableEntry {
            depth,
            score: if alpha <= original_alpha {
                TableScore::UpperBound(alpha)
            } else if alpha >= beta {
                TableScore::LowerBound(alpha)
            } else {
                TableScore::Exact(alpha)
            },
            best_move,
        };

        table.lock().unwrap().insert_entry::<ROOT>(position, entry);
    }

    (alpha, count)
}

pub fn search_single(
    position: &Position,
    guess: ValueScore,
    depth: Depth,
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (Score, usize) {
    let depth = depth.min(MAX_DEPTH);
    let mut position = *position;

    let mut all_count = 0;

    const WINDOW_SIZE: ValueScore = 100;
    let mut lower_bound = guess - WINDOW_SIZE;
    let mut upper_bound = guess + WINDOW_SIZE;

    for cof in 1.. {
        let (score, count) =
            pvs::<true>(&mut position, depth, lower_bound, upper_bound, table.clone(), constraint);
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
        }

        return if score.abs() >= MATE_SCORE.abs() {
            let pv = table.lock().unwrap().get_pv(&position, depth);
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
        };
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::fen::FromFen, search::table::DEFAULT_TABLE_SIZE_MB};

    fn expect_search(
        fen: &str,
        depth: Depth,
        expected_moves: Vec<&str>,
        expected_score: Option<Score>,
    ) {
        let position = Position::from_fen(fen).unwrap();
        let table = Arc::new(Mutex::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB)));
        let mut constraint = SearchConstraint::default();

        let score = search_single(&position, 0, depth, table.clone(), &mut constraint).0;
        let pv = table.lock().unwrap().get_pv(&position, depth);

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
        expect_search(
            "rnb1r2k/pR3Qpp/2p5/4N3/3P3P/2q5/P2p1PP1/5K1R w - - 1 20",
            2,
            vec!["f7e8"],
            Some(Score::Mate(Color::White, 1)),
        );
    }

    #[test]
    fn mate_them_2() {
        expect_search(
            "rnb1r1k1/pR3ppp/2p5/4N3/3P1Q1P/3p4/P4PP1/q4K1R w - - 3 19",
            6,
            vec!["b7b1", "a1b1", "f4c1", "b1c1"],
            Some(Score::Mate(Color::Black, 2)),
        );
    }

    #[test]
    fn mate_us_3() {
        expect_search(
            "rnb1r1k1/pR3ppp/2p5/4N3/3P1Q1P/2qp4/P4PP1/5K1R b - - 2 18",
            7,
            vec!["c3a1", "b7b1", "a1b1", "f4c1", "b1c1"],
            Some(Score::Mate(Color::Black, 3)),
        );
    }
}
