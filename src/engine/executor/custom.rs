use std::thread;

use camel::{moves::gen::perft, position::Position};

pub fn execute_perft(depth: u8, position: &Position) {
    let position = position.clone();
    thread::spawn(move || {
        let time = std::time::Instant::now();
        let (nodes, _) = perft::<true, true, false>(&position, depth);
        let elapsed = time.elapsed().as_millis();
        let mnps = nodes as f64 / 1000.0 / (elapsed + 1) as f64;
        println!("Searched {} nodes in {} ms [{:.3} Mnps]", nodes, elapsed, mnps);
    });
}

pub fn execute_do_move(mov_str: &str, position: &mut Position) {
    if let Some(mov) = position.moves::<false>().iter().find(|mov| mov.to_string() == mov_str) {
        *position = position.make_move(*mov);
    } else {
        println!("Illegal move: {}", mov_str);
    }
}

pub fn execute_display(position: &Position) {
    print!("{}", position.board);
    println!("{}", position.to_fen());
}

pub fn execute_all_moves(position: &Position) {
    let moves = position.moves::<false>();
    for mov in moves {
        print!("{} ", mov);
    }
    println!();
}

pub fn execute_clear() {
    if !std::process::Command::new("clear").status().unwrap().success() {
        std::process::Command::new("cls");
    }
}

pub fn execute_quit() {
    std::process::exit(0);
}
