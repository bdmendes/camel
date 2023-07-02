use self::{constraint::SearchConstraint, table::SearchTable};
use crate::{evaluation::Score, position::Position};

pub mod constraint;
pub mod pvs;
pub mod table;

pub type Depth = i16;

fn print_iter_info(
    position: &Position,
    depth: Depth,
    score: Score,
    count: usize,
    elapsed: u128,
    table: &mut SearchTable,
) {
    let nps = (count as f64 / ((elapsed + 1) as f64 / 1000.0)) as usize;
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

    print!("time {} nodes {} nps {} pv", elapsed, count, nps);

    let pv = table.get_pv(position, depth);
    for mov in pv {
        print!(" {}", mov);
    }
    println!();
}

pub fn search_iter(
    position: &Position,
    depth: Depth,
    table: &mut SearchTable,
    constraint: &mut SearchConstraint,
) {
    let one_legal_move = position.moves::<false>().len() == 1;

    for d in 1..=depth {
        let time = std::time::Instant::now();
        let (score, count) = pvs::search(position, d, table, constraint);

        if constraint.should_stop_search() {
            break;
        }

        let elapsed = time.elapsed();
        print_iter_info(position, d, score, count, elapsed.as_millis(), table);

        if one_legal_move || matches!(score, Score::Mate(_, _)) {
            break;
        }

        if elapsed > constraint.remaining_time().unwrap_or_else(|| elapsed) {
            break;
        }
    }

    let best_move = table.get_hash_move(position);
    if let Some(mov) = best_move {
        println!("bestmove {}", mov);
    }
}
