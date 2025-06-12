use crate::{
    evaluation::Evaluable,
    position::{
        fen::{FromFen, ToFen},
        Position,
    },
    search::{
        constraint::SearchConstraint, history::BranchHistory, pvs::pvs, table::SearchTable, Depth,
    },
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fs::{read_to_string, OpenOptions},
    io::Write,
    sync::Arc,
};

const EVAL_DEPTH: Depth = 7;

pub fn label_quiet_epd() {
    let positions: Vec<Position> = {
        let epd_file =
            read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/books/quiet-labeled.epd"))
                .expect("Could not read file");
        epd_file
            .lines()
            .collect::<Vec<&str>>()
            .par_iter()
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let fen = parts.iter().take(4).cloned().collect::<Vec<&str>>().join(" ");
                Position::from_fen(&fen).unwrap()
            })
            .collect()
    };

    let table = Arc::new(SearchTable::new(2048));
    let constraint = SearchConstraint::default();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(concat!(env!("CARGO_MANIFEST_DIR"), "/books/quiet-evaluated-filtered-camelv1.epd"))
        .unwrap();

    let mut counter: u64 = 0;
    let data: HashMap<String, i16> = positions
        .iter()
        .filter_map(|p| {
            let static_eval = p.value();
            let eval = pvs::<true, true, false>(
                &mut p.clone(),
                EVAL_DEPTH,
                -20000,
                20000,
                table.clone(),
                &constraint,
                &mut BranchHistory(Vec::new()),
                0,
            )
            .0 * p.side_to_move.sign();
            let diff = eval.saturating_sub(static_eval).unsigned_abs();
            let fen = p.to_fen();

            counter += 1;

            if diff < 200 && eval.abs() < 2000 {
                println!("[{}] fen: {}, eval: {}, diff: {}", counter, fen, eval, diff);
                Some((fen, eval))
            } else {
                None
            }
        })
        .collect();

    for (fen, eval) in data {
        writeln!(&mut file, "{} cp \"{}\";", fen, eval).unwrap();
    }
}
