use crate::{
    evaluation::{evaluate_move, evaluate_position, Score, ValueScore},
    position::{Color, Position},
};

use self::table::SearchTable;

pub mod table;

pub type Depth = i16;

fn quiesce(position: &Position, mut alpha: ValueScore, beta: ValueScore) -> ValueScore {
    let stand_pat = evaluate_position(position);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let moves = position.moves::<true>();

    for mov in moves.iter() {
        let score = -quiesce(&position.make_move(*mov), -beta, -alpha);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn alphabeta_internal(
    position: &Position,
    depth: Depth,
    mut alpha: ValueScore,
    beta: ValueScore,
    table: &mut SearchTable,
) -> ValueScore {
    let is_check = position.is_check();

    if depth <= 0 && !is_check {
        return quiesce(position, alpha, beta);
    }

    let mut moves = position.moves::<false>();
    if moves.is_empty() {
        return if is_check { ValueScore::MIN + 1 + depth } else { 0 };
    }

    let hash_move = table.get_hash_move(position);
    moves.sort_unstable_by_key(move |mov| {
        if hash_move.is_some() && mov == &hash_move.unwrap() {
            return ValueScore::MAX;
        }
        evaluate_move(position, *mov)
    });

    let mut best_move = moves[0];

    for mov in moves.iter() {
        let score = -alphabeta_internal(
            &position.make_move(*mov),
            if is_check { depth } else { depth - 1 },
            -beta,
            -alpha,
            table,
        );

        if score >= beta {
            return beta;
        }
        if score > alpha {
            best_move = *mov;
            alpha = score;
        }
    }

    table.insert_hash_move(*position, best_move, depth);

    alpha
}

pub fn alphabeta(position: &Position, depth: Depth, table: &mut SearchTable) -> Score {
    let score = alphabeta_internal(position, depth, ValueScore::MIN + 1, ValueScore::MAX, table);
    if score.abs() > ValueScore::MAX - depth - 1 {
        Score::Mate(
            if score > 0 { Color::White } else { Color::Black },
            ((ValueScore::MAX - score.abs()) / 2) as u8,
        )
    } else {
        Score::Value(score)
    }
}
