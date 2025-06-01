use dataset::PositionScore;
use rand::seq::SliceRandom;

use super::nnue::{NeuralNetwork, HIDDEN_LAYER_SIZE, INPUT_SIZE, MAX_CLAMP};
use crate::core::{color::Color, piece::Piece, square::Square, Position};

pub mod dataset;

fn input_value(position: &Position, index: usize) -> i16 {
    let square = Square::from((index % 64) as u8).unwrap();
    let piece = Piece::from(((index / 64) % 6) as u8).unwrap();
    let color = Color::from((index / (64 * 6)) as u8).unwrap();
    debug_assert!((color as usize) * 64 * 6 + (piece as usize) * 64 + square as usize == index);
    match position.piece_color_at(square) {
        Some((p, c)) if p == piece && c == color => 1,
        _ => 0,
    }
}

fn backpropagate(net: &mut NeuralNetwork, position: &PositionScore, learning_rate: f32) -> f32 {
    let output = net.evaluate(&position.position) as f32;
    let target = position.result.to_score() as f32;
    let error = output - target;
    let loss = error * error;

    // Gradients for output layer
    for i in 0..HIDDEN_LAYER_SIZE {
        let activation = (net.acc[i] + net.params.acc_biases[i]).clamp(0, MAX_CLAMP);
        let grad_out_weight = error * activation as f32;
        net.params.out_weights[i] -= (grad_out_weight * learning_rate) as i32;
    }
    net.params.out_bias -= (error * learning_rate) as i16;

    // Gradients for hidden layer
    for i in 0..HIDDEN_LAYER_SIZE {
        let activation = net.acc[i] + net.params.acc_biases[i];
        if activation > 0 {
            let delta = error * net.params.out_weights[i] as f32;
            for j in 0..INPUT_SIZE {
                let input = input_value(&position.position, j) as f32;
                let grad_acc_weight = delta * input;
                net.params.acc_weights[j * HIDDEN_LAYER_SIZE + i] -=
                    (grad_acc_weight * learning_rate) as i32;
            }
            net.params.acc_biases[i] -= (delta * learning_rate) as i32;
        }
    }

    loss
}

pub fn train_nnue(
    net: &mut NeuralNetwork,
    dataset: &[PositionScore],
    learning_rate: f32,
    epochs: usize,
) {
    let mut rng = rand::thread_rng();

    for epoch in 0..epochs {
        let mut shuffled_dataset = dataset.to_vec();
        shuffled_dataset.shuffle(&mut rng);

        let adjusted_lr = learning_rate * (0.99_f32).powi(epoch as i32 / 10);

        let mut total_loss: f32 = 0.0;
        for (idx, position) in shuffled_dataset.iter().enumerate() {
            let loss = backpropagate(net, position, adjusted_lr);
            total_loss += loss;

            if idx % 10_000 == 0 {
                println!(
                    "Epoch {} [{}%]: Processed {}/{} positions [{}%]; online loss: {}",
                    epoch + 1,
                    (epoch + 1) * 100 / epochs,
                    idx + 1,
                    dataset.len(),
                    (idx + 1) * 100 / dataset.len(),
                    total_loss / (idx + 1) as f32
                );
            }
        }

        let average_loss = total_loss / dataset.len() as f32;

        println!(
            "Epoch {}; Learning rate: {}, Average Loss = {}\n",
            epoch + 1,
            adjusted_lr,
            average_loss,
        );
    }
}
