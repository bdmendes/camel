use super::{constraint::SearchConstraint, movepick::MovePicker, see, Depth};
use crate::{
    evaluation::{position::MAX_POSITIONAL_GAIN, Evaluable, ValueScore, MATE_SCORE},
    position::{board::Piece, Position},
};

pub fn quiesce(
    position: &Position,
    mut alpha: ValueScore,
    beta: ValueScore,
    constraint: &SearchConstraint,
    ply: Depth,
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
        let score = if is_check { MATE_SCORE + ply as ValueScore } else { static_evaluation };
        return (score, 1);
    }

    let mut count = 1;

    for mov in picker {
        if !is_check && mov.flag().is_capture() {
            // Delta pruning: this capture cannot improve the score in any way.
            let captured_piece = position.board.piece_at(mov.to()).unwrap_or(Piece::Pawn);
            if static_evaluation + captured_piece.value() + MAX_POSITIONAL_GAIN < alpha {
                continue;
            }

            // Static exchange evaluation: if we lose material, there is no point in searching further.
            if see::see::<true>(mov, &position.board) < 0 {
                continue;
            }
        }

        let (score, nodes) =
            quiesce(&position.make_move(mov), -beta, -alpha, constraint, ply.saturating_add(1));
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
