use std::str::FromStr;

use crate::{
    core::position::{Position, PositionDiffEntry, color::Color, piece::Piece, square::Square},
    evaluation::ValueScore,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

// 2 sides, 6 pieces, 64 squares.
pub const INPUT_SIZE: usize = 768;

// We have a single hidden layer in our network.
pub const HIDDEN_LAYER_SIZE: usize = 128;

// The actual NN output is -1 to 1, to improve training dynamics.
pub const SCALE: f32 = 2000.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameters {
    // The "accumulator" is the cached input of the hidden layer.
    // In practice, it will be 0 (empty) or 1 (set) times the weights.
    pub acc_weights: Vec<f32>,
    pub acc_biases: Vec<f32>,

    // The output of the hidden layer is fed to the "output"
    // parameters to generate the final static evaluation.
    pub out_weights: Vec<f32>,
    pub out_bias: f32,
}

impl Parameters {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let acc_weights = (0..INPUT_SIZE * HIDDEN_LAYER_SIZE)
            .map(|_| rng.random_range(-1.0..1.0))
            .collect();
        let acc_biases = (0..HIDDEN_LAYER_SIZE)
            .map(|_| rng.random_range(-1.0..1.0))
            .collect();
        let out_weights = (0..HIDDEN_LAYER_SIZE)
            .map(|_| rng.random_range(-1.0..1.0))
            .collect();
        let out_bias = rng.random_range(-1.0..1.0);
        Self {
            acc_weights,
            acc_biases,
            out_weights,
            out_bias,
        }
    }

    pub fn filled(
        acc_weight_val: f32,
        acc_bias_val: f32,
        out_weight_val: f32,
        out_bias_val: f32,
    ) -> Self {
        Self {
            acc_weights: vec![acc_weight_val; INPUT_SIZE * HIDDEN_LAYER_SIZE],
            acc_biases: vec![acc_bias_val; HIDDEN_LAYER_SIZE],
            out_weights: vec![out_weight_val; HIDDEN_LAYER_SIZE],
            out_bias: out_bias_val,
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, self).map_err(std::io::Error::other)
    }
}

impl FromStr for Parameters {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

pub struct NeuralNetwork {
    pub params: Parameters,
    pub acc: Vec<f32>,
    pub last_seen: Option<(Position, f32)>,
}

impl NeuralNetwork {
    pub fn new(params: Parameters) -> Self {
        Self {
            params,
            acc: vec![0.0; HIDDEN_LAYER_SIZE],
            last_seen: None,
        }
    }

    fn acc_index(piece: Piece, color: Color, square: Square) -> usize {
        (color as usize) * 64 * 6 + (piece as usize) * 64 + square as usize
    }

    pub fn activate(value: f32) -> f32 {
        // Regular ReLU.
        value.max(0.0)
    }

    fn set(&mut self, piece: Piece, color: Color, square: Square) {
        let idx = Self::acc_index(piece, color, square);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[i] += self.params.acc_weights[idx * HIDDEN_LAYER_SIZE + i];
        }
    }

    fn clear(&mut self, piece: Piece, color: Color, square: Square) {
        let idx = Self::acc_index(piece, color, square);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[i] -= self.params.acc_weights[idx * HIDDEN_LAYER_SIZE + i];
        }
    }

    fn forward(&self) -> f32 {
        let mut eval: f32 = 0.0;

        for i in 0..HIDDEN_LAYER_SIZE {
            let hidden_out = Self::activate(self.acc[i] + self.params.acc_biases[i]);
            eval += hidden_out * self.params.out_weights[i];
        }

        eval + self.params.out_bias
    }

    fn forward_and_cache(&mut self, position: &Position) -> f32 {
        let res = self.forward();
        self.last_seen = Some((*position, res));
        res
    }

    fn evaluate_unscaled(&mut self, position: &Position) -> f32 {
        match self.last_seen {
            Some((last_seen, score)) if last_seen.hash() == position.hash() => score,
            Some((last_seen, _)) => {
                let diff = position.diff(&last_seen);
                for e in diff {
                    match e {
                        PositionDiffEntry::Set(square, piece, color) => {
                            self.set(piece, color, square);
                        }
                        PositionDiffEntry::Clear(square, piece, color) => {
                            self.clear(piece, color, square);
                        }
                    }
                }
                self.forward_and_cache(position)
            }
            _ => {
                for square in Square::list() {
                    if let Some((piece, color)) = position.piece_color_at(*square) {
                        self.set(piece, color, *square);
                    }
                }
                self.forward_and_cache(position)
            }
        }
    }

    pub fn evaluate(&mut self, position: &Position) -> ValueScore {
        (self.evaluate_unscaled(position) * SCALE) as ValueScore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulator1() {
        // Set all accumulator weights to 1, and biases to 0.
        let params = Parameters::filled(1.0, 0.0, 0.0, 0.0);
        let mut net = NeuralNetwork::new(params);

        // Independently of the square, all accumulator nodes will be fed with 1.
        net.set(Piece::Queen, Color::White, Square::E4);

        net.acc.iter().for_each(|&x| assert_eq!(x, 1.0));
    }

    #[test]
    fn accumulator2() {
        // Set all accumulator weights to 1, except for the White Queen on E4.
        let mut params = Parameters::filled(1.0, 0.0, 0.0, 0.0);

        let queen_e4_index = NeuralNetwork::acc_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[queen_e4_index * HIDDEN_LAYER_SIZE + i] = 2.0;
        }
        let mut net = NeuralNetwork::new(params);

        net.set(Piece::Queen, Color::White, Square::E4);
        net.acc.iter().for_each(|&x| assert_eq!(x, 2.0));

        net.set(Piece::Rook, Color::White, Square::E4);
        net.acc.iter().for_each(|&x| assert_eq!(x, 3.0));

        net.clear(Piece::Queen, Color::White, Square::E4);
        net.acc.iter().for_each(|&x| assert_eq!(x, 1.0));

        net.clear(Piece::Rook, Color::White, Square::E4);
        net.acc.iter().for_each(|&x| assert_eq!(x, 0.0));
    }

    #[test]
    fn forward() {
        // Set all accumulator weights to 1, and biases to 0.
        let params = Parameters::filled(1.0, 2.0, 1.0, 10.0);
        let mut net = NeuralNetwork::new(params);

        // Set the Queen on E4, which will set all accumulators to 1.
        net.set(Piece::Queen, Color::White, Square::E4);
        assert_eq!(net.forward(), HIDDEN_LAYER_SIZE as f32 * 3.0 + 10.0);

        // Set the Rook on E4, which will add 1 to all accumulators.
        net.set(Piece::Rook, Color::White, Square::E4);
        assert_eq!(net.forward(), HIDDEN_LAYER_SIZE as f32 * 4.0 + 10.0);
    }

    #[test]
    fn evaluate() {
        // Set all weights to 1, except for the White Queen on E4.
        let mut params = Parameters::filled(1.0, 0.0, 1.0, 0.0);

        let queen_e4_index = NeuralNetwork::acc_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[queen_e4_index * HIDDEN_LAYER_SIZE + i] = 2.0;
        }
        let mut net = NeuralNetwork::new(params);

        assert_eq!(net.last_seen, None);

        let mut position = Position::default();
        position.set_square(Square::E4, Piece::Queen, Color::White);

        assert_eq!(
            net.evaluate_unscaled(&position),
            2.0 * HIDDEN_LAYER_SIZE as f32
        );

        assert_eq!(
            net.last_seen,
            Some((position, 2.0 * HIDDEN_LAYER_SIZE as f32))
        );

        assert_eq!(
            net.evaluate_unscaled(&position),
            2.0 * HIDDEN_LAYER_SIZE as f32
        );

        position.clear_square(Square::E4);
        assert_eq!(net.evaluate_unscaled(&position), 0.0);

        position.set_square(Square::E4, Piece::Rook, Color::White);
        assert_eq!(net.evaluate_unscaled(&position), HIDDEN_LAYER_SIZE as f32);
    }
}
