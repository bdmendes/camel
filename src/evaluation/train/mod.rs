use rand::seq::SliceRandom;

use super::nnue::{NeuralNetwork, HIDDEN_LAYER_SIZE};
use crate::{
    core::{color::Color, piece::Piece, square::Square, Position},
    evaluation::nnue::{INPUT_SIZE, SCALE},
};

pub mod dataset;

fn input_value(position: &Position, index: usize) -> f32 {
    let square = Square::from((index % 64) as u8).unwrap();
    let piece = Piece::from(((index / 64) % 6) as u8).unwrap();
    let color = Color::from((index / (64 * 6)) as u8).unwrap();
    match position.piece_color_at(square) {
        Some((p, c)) if p == piece && c == color => 1.0,
        _ => 0.0,
    }
}

fn backpropagate(net: &mut NeuralNetwork, position: &(Position, i16), learning_rate: f32) -> f32 {
    // Since we're changing the weights in this process, we cannot cache the accumulator
    // as we'll do in the regular NN feedforward during search.
    net.reset();

    // Our evaluation artifically scales itself to yeild a centipawn-like value.
    // Our target is -1 to 1.
    let output = (net.evaluate(&position.0) as f32) / SCALE;
    let target = (position.1 as f32 / SCALE).clamp(-1.0, 1.0);
    let error = output - target;

    for i in 0..HIDDEN_LAYER_SIZE {
        let hidden_activation = NeuralNetwork::activate(net.acc[i] + net.params.acc_biases[i]);
        let weight = net.params.out_weights[i];

        // Compute gradients for the output weights.
        {
            let delta = hidden_activation * error;
            net.params.out_weights[i] -= delta * learning_rate;
        }

        // Compute gradients for the accumulator weights.
        if hidden_activation > 0.0 {
            let delta = error * weight;
            for j in 0..INPUT_SIZE {
                net.params.acc_weights[j * HIDDEN_LAYER_SIZE + i] -=
                    delta * input_value(&position.0, j) * learning_rate;
            }
            net.params.acc_biases[i] -= delta * learning_rate;
        }
    }

    net.params.out_bias -= error * learning_rate;

    error
}

pub fn train_nnue(
    net: &mut NeuralNetwork,
    dataset: &[(Position, i16)],
    learning_rate: f32,
    epochs: usize,
) {
    let mut rng = rand::thread_rng();

    for epoch in 0..epochs {
        let mut shuffled_dataset = dataset.to_vec();
        shuffled_dataset.shuffle(&mut rng);

        let adjusted_lr = learning_rate * (0.99_f32).powi(epoch as i32 / 10);

        let mut total_error: f32 = 0.0;
        for (idx, position) in shuffled_dataset.iter().enumerate() {
            let error = backpropagate(net, position, adjusted_lr);
            total_error += error.abs();

            if idx % 10_000 == 0 {
                println!(
                    "Epoch {} [{}%]: Processed {}/{} positions [{}%]; online loss: {}",
                    epoch + 1,
                    (epoch + 1) * 100 / epochs,
                    idx + 1,
                    dataset.len(),
                    (idx + 1) * 100 / dataset.len(),
                    total_error * SCALE / (idx + 1) as f32
                );
            }
        }

        let average_loss = total_error * SCALE / dataset.len() as f32;

        println!(
            "Epoch {}; Learning rate: {}, Average Loss = {}\n",
            epoch + 1,
            adjusted_lr,
            average_loss,
        );

        net.params
            .save("assets/models/nnue-quiet-labeled.bin")
            .expect("Failed to save NNUE parameters");
    }
}
