use super::{
    constraint::SearchConstraint,
    movepick::MovePicker,
    table::{SearchTable, TableEntry, TableScore},
    Depth, MAX_DEPTH,
};
use crate::{
    evaluation::{
        moves::evaluate_move,
        piece_value,
        position::{evaluate_position, MAX_POSITIONAL_GAIN},
        Score, ValueScore,
    },
    position::{board::Piece, Color, Position},
};
use std::sync::{Arc, Mutex};

const MIN_SCORE: ValueScore = ValueScore::MIN + 1;
const MAX_SCORE: ValueScore = -MIN_SCORE;
const MATE_SCORE: ValueScore = ValueScore::MIN + MAX_DEPTH as ValueScore + 1;

const NULL_MOVE_REDUCTION: Depth = 3;

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
    // so we must search all check evasion. Otherwise, search only captures
    let is_check = position.is_check();
    let static_evaluation = if is_check {
        alpha
    } else {
        let static_evaluation = evaluate_position(position) * position.side_to_move.sign();

        // Standing pat: captures are not forced
        alpha = alpha.max(static_evaluation);

        // Beta cutoff: position is too good
        if static_evaluation >= beta {
            return (beta, 1);
        }

        // Delta pruning: sequence cannot improve the score
        if static_evaluation < alpha.saturating_sub(piece_value(Piece::Queen)) {
            return (alpha, 1);
        }

        static_evaluation
    };

    let moves = position.moves(!is_check);
    let picker = MovePicker::new(&moves, |m| evaluate_move(position, m));

    // Stable position reached
    if moves.is_empty() {
        let score = if is_check { MATE_SCORE } else { static_evaluation };
        return (score, 1);
    }

    let mut count = 1;

    for (mov, _, _) in picker {
        // Delta prune move if it cannot improve the score
        if !is_check && mov.flag().is_capture() {
            let captured_piece =
                position.board.piece_color_at(mov.to()).map_or_else(|| Piece::Pawn, |p| p.0);
            if static_evaluation + piece_value(captured_piece) + MAX_POSITIONAL_GAIN < alpha {
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
    position: &Position,
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
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (ValueScore, usize) {
    if ROOT {
        table.lock().unwrap().prepare_for_new_search(position.fullmove_number);
    }

    let twofold_repetition = constraint.is_repetition::<2>(position);
    let threefold_repetition = twofold_repetition && constraint.is_repetition::<3>(position);

    if !ROOT {
        // Detect history-related draws
        if position.halfmove_clock >= 100 || threefold_repetition {
            return (0, 1);
        }

        // Get known score from transposition table
        if !twofold_repetition {
            if let Some(tt_entry) = table.lock().unwrap().get_table_score(position, depth) {
                match tt_entry {
                    TableScore::Exact(score) if score.abs() < MATE_SCORE.abs() => {
                        return (score, 1)
                    }
                    TableScore::LowerBound(score) => alpha = alpha.max(score),
                    TableScore::UpperBound(score) => beta = beta.min(score),
                    _ => (),
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
    if !ROOT
        && !is_check
        && depth > NULL_MOVE_REDUCTION
        && position.board.piece_count(Color::White) > 0
        && position.board.piece_count(Color::Black) > 0
        && !twofold_repetition
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

    let moves = position.moves(false);

    // Detect checkmate and stalemate
    if moves.is_empty() {
        let score = if is_check { MATE_SCORE - depth as ValueScore } else { 0 };
        return (score, count);
    }

    // Sort moves via MVV-LVA, psqt and table information
    let hash_move = table.lock().unwrap().get_hash_move(position);
    let killer_moves = table.lock().unwrap().get_killers(depth);
    let picker = MovePicker::new(&moves, |mov| {
        if hash_move.is_some() && mov == hash_move.unwrap() {
            return ValueScore::MAX;
        }
        if Some(mov) == killer_moves[0] || Some(mov) == killer_moves[1] {
            return piece_value(Piece::Queen);
        }
        evaluate_move(position, mov)
    });

    let original_alpha = alpha;
    let mut best_move = moves[0];

    for (mov, _, i) in picker {
        let new_position = position.make_move(mov);

        constraint.visit_position(&new_position, mov.flag().is_reversible());
        let (score, nodes) =
            pvs_recurse(&new_position, depth, alpha, beta, table.clone(), constraint, i > 0);
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

pub fn search(
    position: &Position,
    depth: Depth,
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
) -> (Score, usize) {
    let depth = depth.min(MAX_DEPTH);

    let (score, count) =
        pvs::<true>(position, depth, MIN_SCORE, MAX_SCORE, table.clone(), constraint);

    if score.abs() >= MATE_SCORE.abs() {
        let pv = table.lock().unwrap().get_pv(position, depth);
        let plys_to_mate = pv.len() as u8;
        (
            Score::Mate(
                if score > 0 { position.side_to_move } else { position.side_to_move.opposite() },
                (plys_to_mate + 1) / 2,
            ),
            count,
        )
    } else {
        (Score::Value(score), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::table::DEFAULT_TABLE_SIZE_MB;

    fn expect_search(
        fen: &str,
        depth: Depth,
        expected_moves: Vec<&str>,
        expected_score: Option<Score>,
    ) {
        let position = Position::from_fen(fen).unwrap();
        let table = Arc::new(Mutex::new(SearchTable::new(DEFAULT_TABLE_SIZE_MB)));
        let mut constraint = SearchConstraint::default();

        let score = search(&position, depth, table.clone(), &mut constraint).0;
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
    fn mate_them_1() {
        expect_search(
            "rnb1r1k1/pR3Qpp/2p5/4N3/3P3P/2q5/P2p1PP1/5K1R b - - 0 19",
            2,
            vec!["g8h8", "f7e8"],
            Some(Score::Mate(Color::White, 1)),
        );
    }

    #[test]
    fn mate_them_2() {
        expect_search(
            "rnb1r1k1/pR3ppp/2p5/4N3/3P1Q1P/3p4/P4PP1/q4K1R w - - 3 19",
            7,
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

    #[test]
    fn mate_us_5() {
        expect_search(
            "4R3/1R2b1p1/2B2k2/N4p2/1P6/P1K3P1/5P2/8 w - - 2 41",
            9,
            vec!["e8e7"],
            Some(Score::Mate(Color::White, 5)),
        )
    }
}
