use core::{fen::START_POSITION, Position};
use std::str::FromStr;

use evaluation::{
    nnue::{NeuralNetwork, Parameters},
    train::{dataset::load_scored_epd, train_nnue},
};

#[allow(dead_code)]
mod core;
#[allow(dead_code)]
mod evaluation;
#[allow(dead_code)]
mod search;

fn main() {
    let dataset: Vec<_> = load_scored_epd("assets/books/quiet-evaluated-filtered-camelv1.epd");
    println!("Loaded {} positions from dataset", dataset.len());

    let params = Parameters::random();

    let mut net = NeuralNetwork::new(params);

    let learning_rate = 0.01;
    let epochs = 100;

    println!("Equal: {}", net.evaluate(&Position::from_str(START_POSITION).unwrap()));
    println!(
        "Black better: {}",
        net.evaluate(&Position::from_str("4r3/8/8/5p2/5k2/8/K7/6n1 b - - 0 1").unwrap())
    );

    train_nnue(&mut net, &dataset, learning_rate, epochs);

    println!("Equal: {}", net.evaluate(&Position::from_str(START_POSITION).unwrap()));
    println!(
        "Black better: {}",
        net.evaluate(&Position::from_str("4r3/8/8/5p2/5k2/8/K7/6n1 b - - 0 1").unwrap())
    );

    net.params
        .save("assets/models/nnue-quiet-labeled.bin")
        .expect("Failed to save NNUE parameters");
}
