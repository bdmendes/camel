use self::{constraint::SearchConstraint, table::SearchTable};
use crate::{
    evaluation::{Score, ValueScore},
    moves::gen::MoveStage,
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

    print!(
        "time {} nodes {} nps {} hashfull {} pv",
        (elapsed_micros / 1000).max(1),
        count,
        nps,
        table.hashfull_millis()
    );

    let pv = table.get_pv(position, depth);
    for mov in pv {
        print!(" {}", mov);
    }

    println!();
}

pub fn search_iterative_deepening_multithread(
    position: &Position,
    mut current_guess: ValueScore,
    depth: Depth,
    table: Arc<SearchTable>,
    constraint: &SearchConstraint,
) {
    let moves = position.moves(MoveStage::All);

    if moves.is_empty() {
        return;
    }

    let one_legal_move = moves.len() == 1;
    let number_threads = constraint.number_threads.load(std::sync::atomic::Ordering::Relaxed);

    let mut current_depth = 1;

    while constraint.pondering() || current_depth <= depth {
        let time = std::time::Instant::now();

        let (score, count) = thread::scope(|s| {
            // Start helper threads.
            let handles = (1..number_threads)
                .map(|_| {
                    let table = table.clone();
                    s.spawn(move || {
                        pvs::pvs_aspiration::<false>(
                            position,
                            current_guess,
                            current_depth,
                            table,
                            constraint,
                        )
                    })
                })
                .collect::<Vec<_>>();

            // Run the main thread.
            let (score, count) = pvs::pvs_aspiration::<true>(
                position,
                current_guess,
                current_depth,
                table.clone(),
                constraint,
            );

            // Signal helper threads to stop and join them.
            constraint.threads_stop.store(true, Ordering::Relaxed);
            for handle in handles {
                handle.join().unwrap();
            }
            constraint.threads_stop.store(false, Ordering::Release);

            // Use the main thread result.
            (score, count)
        });

        if constraint.should_stop_search() {
            break;
        }

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

        current_depth = current_depth.saturating_add(1);
    }

    // Best move found
    let best_move = table.get_hash_move(position).unwrap_or(moves[0]);
    print!("bestmove {}", best_move);

    // Ponder move if possible
    let new_position = position.make_move(best_move);
    if let Some(ponder_move) = table.get_hash_move(&new_position) {
        println!(" ponder {}", ponder_move);
    } else {
        println!();
    }
}
