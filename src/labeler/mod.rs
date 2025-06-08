use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    sync::Arc,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    position::{
        fen::{FromFen, ToFen},
        Color, Position,
    },
    search::{constraint::SearchConstraint, pvs::pvs_aspiration, table::SearchTable, Depth},
};

const EVAL_DEPTH: Depth = 6;

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

    let table = Arc::new(SearchTable::new(256));
    let constraint = SearchConstraint::default();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(concat!(env!("CARGO_MANIFEST_DIR"), "/books/quiet-evaluated-camelv1.epd"))
        .unwrap();

    positions.iter().enumerate().for_each(|(i, p)| {
        let eval = {
            let ours =
                pvs_aspiration::<false>(p, 0, EVAL_DEPTH, table.clone(), &constraint).unwrap().0;
            match p.side_to_move {
                Color::White => ours.cp(),
                Color::Black => -ours.cp(),
            }
        };
        println!("[{}] {} cp \"{}\";", i, p.to_fen(), eval);
        writeln!(&mut file, "{} cp \"{}\";", p.to_fen(), eval).unwrap();
    });
}
