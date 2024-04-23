use self::{constraint::SearchConstraint, table::SearchTable};
use crate::{
    evaluation::{moves::evaluate_move, Score, ValueScore},
    moves::{gen::MoveStage, Move},
    position::Position,
};
use std::{
    sync::{atomic::Ordering, Arc},
    thread::{self},
    time::Duration,
};

pub mod constraint;
pub mod history;
pub mod movepick;
pub mod pvs;
pub mod see;
pub mod table;

pub type Depth = u8;

pub const MAX_DEPTH: Depth = 50;

fn print_iter_info(
    position: &Position,
    depth: Depth,
    score: Score,
    count: usize,
    elapsed: Duration,
    table: &SearchTable,
) {
    let elapsed_micros = elapsed.as_micros();
    let nps = (count as f64 / (elapsed_micros.max(1) as f64 / 1000000.0)) as usize;

    print!("info depth {} ", depth);

    match score {
        Score::Value(score) => {
            print!("score cp {} ", score);
        }
        Score::Mate(color, moves) => {
            if color == position.side_to_move {
                print!("score mate {} ", moves);
            } else {
                let moves = moves as i16;
                print!("score mate {} ", -moves);
            }
        }
    }

    println!(
        "time {} nodes {} nps {} hashfull {} pv {}",
        (elapsed_micros / 1000).max(1),
        count,
        nps,
        table.hashfull_millis(),
        table.get_pv(position, depth).iter().map(|m| m.to_string()).collect::<Vec<_>>().join(" ")
    );
}

pub fn search_iterative_deepening_multithread(
    position: &Position,
    mut current_guess: ValueScore,
    depth: Depth,
    table: Arc<SearchTable>,
    constraint: &SearchConstraint,
) -> Option<Move> {
    let mut moves = position.moves(MoveStage::All);

    if moves.is_empty() {
        return None;
    }

    let one_legal_move = moves.len() == 1;
    let number_threads = constraint.number_threads.load(std::sync::atomic::Ordering::Relaxed);

    let mut current_depth = 1;

    while constraint.pondering() || current_depth <= depth {
        let time = std::time::Instant::now();

        let search_result = thread::scope(|s| {
            // We must tell threads that it is ok to run.
            constraint.threads_stop.store(false, Ordering::Relaxed);

            // It is important to at least get a move with depth == 1, so do the simplest thing possible.
            let multi_thread = number_threads > 1 && current_depth > 1;

            if !multi_thread {
                // Simply return a single threaded result.
                return pvs::pvs_aspiration::<true>(
                    position,
                    current_guess,
                    current_depth,
                    table.clone(),
                    constraint,
                );
            }

            // Start threads.
            // The main thread will signal others to stop.
            let handles = (0..number_threads)
                .map(|i| {
                    let table = table.clone();
                    let pvs_function = if i == 0 {
                        pvs::pvs_aspiration::<true>
                    } else {
                        pvs::pvs_aspiration::<false>
                    };
                    s.spawn(move || {
                        pvs_function(position, current_guess, current_depth, table, constraint)
                    })
                })
                .collect::<Vec<_>>();

            // Wait for the threads to stop and return the result of the main thread.
            let results = handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>();
            results[0]
        });

        if search_result.is_none() {
            // The search could not finish in time.
            break;
        }

        let (score, count) = search_result.unwrap();

        if let Score::Value(score) = score {
            current_guess = score;
        }

        let elapsed = time.elapsed();
        print_iter_info(position, current_depth, score, count, time.elapsed(), &table);

        if !constraint.pondering()
            && (one_legal_move
                || matches!(score, Score::Mate(_, _))
                || elapsed > constraint.remaining_time().unwrap_or(elapsed))
        {
            break;
        }

        current_depth = (current_depth + 1).min(MAX_DEPTH);
    }

    if let Some(best_move) = table.get_hash_move(position) {
        // Best move found, as expected.
        print!("bestmove {}", best_move);

        // Tell operator we'd like to ponder on this next move next, while the opponent is thinking.
        let new_position = position.make_move(best_move);
        if let Some(ponder_move) = table.get_hash_move(&new_position) {
            println!(" ponder {}", ponder_move);
        } else {
            println!();
        }

        Some(best_move)
    } else {
        // We are in time trouble. Return a "panic" perceived best move.
        moves.sort_by_cached_key(|m| -evaluate_move(position, *m));
        println!("bestmove {}", moves[0]);

        Some(moves[0])
    }
}
