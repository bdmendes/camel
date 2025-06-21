use core::Position;
use std::str::FromStr;

use evaluation::nnue::{NeuralNetwork, Parameters};

#[allow(dead_code)]
mod core;
#[allow(dead_code)]
mod evaluation;
#[allow(dead_code)]
mod search;

fn main() {
    let params = Parameters::load("assets/models/nnue-quiet-labeled.bin").unwrap();
    let mut net = NeuralNetwork::new(params);

    // REPL: read fen and evaluate position
    let mut input = String::new();
    loop {
        println!("Enter FEN (or 'exit' to quit):");
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "exit" {
            break;
        }

        match Position::from_str(input) {
            Ok(position) => {
                let evaluation = net.evaluate(&position);
                println!("Evaluation: {}", evaluation);
            }
            Err(_) => {
                println!("Error parsing FEN.");
            }
        }
    }
}
