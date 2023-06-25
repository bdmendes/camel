use std::{process::Command, thread};

use camel::{
    moves::gen::perft,
    position::{fen::START_FEN, Position},
};

mod uci;

struct Engine {
    pub position: Position,
}

impl Engine {
    fn do_perft(&self, depth: u8) {
        let position = self.position.clone();
        thread::spawn(move || {
            let time = std::time::Instant::now();
            let (nodes, _) = perft::<true, true, false>(&position, depth);
            let elapsed = time.elapsed().as_millis();
            println!(
                "Depth {}: {} in {} ms [{:.3} Mnps]",
                depth,
                nodes,
                elapsed,
                nodes as f64 / 1000.0 / (elapsed + 1) as f64
            );
        });
    }

    fn do_move(&mut self, mov_str: &str) {
        let actual_moves = self.position.moves::<false>();
        if let Some(mov) = actual_moves.iter().find(|mov| mov.to_string() == mov_str) {
            self.position = self.position.make_move(*mov);
        } else {
            println!("Illegal move: {}", mov_str);
        }
    }
}

pub fn uci_loop() {
    let mut engine = Engine { position: Position::from_fen(START_FEN).unwrap() };

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match uci::parse_uci_command(input) {
            Ok(uci::UCICommand::Position { position }) => {
                engine.position = position;
            }
            Ok(uci::UCICommand::Perft { depth }) => engine.do_perft(depth),
            Ok(uci::UCICommand::DoMove { mov_str }) => engine.do_move(&mov_str),
            Ok(uci::UCICommand::Display) => {
                print!("{}", engine.position.board);
                println!("{}", engine.position.to_fen());
            }
            Ok(uci::UCICommand::AllMoves) => {
                let actual_moves = engine.position.moves::<false>();
                for mov in actual_moves {
                    print!("{} ", mov);
                }
                println!();
            }
            Ok(uci::UCICommand::Clear) => {
                let _ = Command::new("clear").status().or_else(|_| Command::new("cls").status());
            }
            Ok(uci::UCICommand::Quit) => std::process::exit(0),
            Err(()) => println!("Invalid command: '{}'", input),
        }
    }
}
