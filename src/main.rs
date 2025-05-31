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
    let dataset = load_scored_epd("assets/books/quiet-labeled.epd");
    println!("Loaded {} positions from dataset", dataset.len());

    let params = Parameters::random();
    let mut net = NeuralNetwork::new(params);

    let learning_rate = 0.008;
    let epochs = 10;
    train_nnue(&mut net, &dataset, learning_rate, epochs);
}
