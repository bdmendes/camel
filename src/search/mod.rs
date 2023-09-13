use self::{constraint::SearchConstraint, table::SearchTable};
use crate::{evaluation::Score, position::Position};
use std::sync::{Arc, Mutex};

pub mod constraint;
mod movepick;
pub mod pvs;
pub mod table;

pub type Depth = i16;

pub const MAX_DEPTH: Depth = 25;

fn print_iter_info(
    position: &Position,
    depth: Depth,
    score: Score,
    count: usize,
    elapsed: u128,
    table: &SearchTable,
) {
    let nps = (count as f64 / (elapsed.max(1) as f64 / 1000.0)) as usize;
    print!("info depth {} hashfull {} ", depth, table.hashfull_millis());

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

    print!("time {} nodes {} nps {} pv", elapsed.max(1), count, nps);

    let pv = table.get_pv(position, depth);
    for mov in pv {
        print!(" {}", mov);
    }
    println!();
}

pub fn search_iter(
    position: &Position,
    depth: Depth,
    table: Arc<Mutex<SearchTable>>,
    constraint: &mut SearchConstraint,
) {
    let moves = position.moves::<false>();

    if moves.is_empty() {
        return;
    }

    let one_legal_move = moves.len() == 1;

    let mut current_depth = 1;
    while current_depth <= depth {
        let time = std::time::Instant::now();
        let (score, count) = pvs::search(position, current_depth, table.clone(), constraint);

        if constraint.should_stop_search() {
            break;
        }

        let elapsed = time.elapsed();
        print_iter_info(
            position,
            current_depth,
            score,
            count,
            elapsed.as_millis(),
            &table.lock().unwrap(),
        );

        if one_legal_move
            || matches!(score, Score::Mate(_, _))
            || elapsed > constraint.remaining_time().unwrap_or(elapsed)
        {
            break;
        }

        current_depth += 1;
    }

    println!("bestmove {}", table.lock().unwrap().get_hash_move(position).unwrap_or(moves[0]));
}
