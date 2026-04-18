use std::{array, str::FromStr};

use crate::{
    evaluation::ValueScore,
    position::{Position, PositionDiffEntry, color::Color, fen::START_POSITION, piece::Piece, square::Square},
};
use rand::RngExt;
use serde::{Deserialize, Serialize};

// 2 sides, 6 pieces, 64 squares.
pub const INPUT_SIZE: usize = 768;

// We have a single hidden layer in our network.
pub const HIDDEN_LAYER_SIZE: usize = 32;

// The actual NN output is cp / SCALE, clamped to [-1, 1].
// 1200 is a reasonable "I'm more than a queen up", corresponding to a completely won position.
pub const SCALE: f64 = 1200.0;

// NNUE inputs are mirrored vertically for Black, as the network generalizes better with this symmetry.
type Perspective = Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameters {
    // The "accumulator" is the cached input of the hidden layer.
    // In practice, it will be 0 (empty) or 1 (set) times the weights.
    // f64 is unusual but crucial to minimize the floating point
    // drift effect and stabilize search.
    pub acc_weights: Vec<f64>,
    pub acc_biases: Vec<f64>,

    // The output of the hidden layer is fed to the "output"
    // parameters to generate the final static evaluation.
    pub out_weights: Vec<f64>,
    pub out_bias: f64,
}

impl Parameters {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let acc_weights = (0..INPUT_SIZE * HIDDEN_LAYER_SIZE)
            .map(|_| rng.random_range(-1.0..1.0))
            .collect();
        let acc_biases = (0..HIDDEN_LAYER_SIZE).map(|_| rng.random_range(-1.0..1.0)).collect();
        let out_weights = (0..HIDDEN_LAYER_SIZE).map(|_| rng.random_range(-1.0..1.0)).collect();
        let out_bias = rng.random_range(-1.0..1.0);
        Self {
            acc_weights,
            acc_biases,
            out_weights,
            out_bias,
        }
    }

    pub fn filled(acc_weight_val: f64, acc_bias_val: f64, out_weight_val: f64, out_bias_val: f64) -> Self {
        Self {
            acc_weights: vec![acc_weight_val; INPUT_SIZE * HIDDEN_LAYER_SIZE],
            acc_biases: vec![acc_bias_val; HIDDEN_LAYER_SIZE],
            out_weights: vec![out_weight_val; HIDDEN_LAYER_SIZE],
            out_bias: out_bias_val,
        }
    }

    fn valid_size(&self) -> bool {
        self.acc_weights.len() == INPUT_SIZE * HIDDEN_LAYER_SIZE
            && self.acc_biases.len() == HIDDEN_LAYER_SIZE
            && self.out_weights.len() == HIDDEN_LAYER_SIZE
    }
}

impl FromStr for Parameters {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<Self>(s) {
            Ok(params) if params.valid_size() => Ok(params),
            Ok(_) => Err("Invalid sizes in NNUE parameters.".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct NeuralNetwork {
    params: Parameters,
    acc: [Vec<f64>; 2],
    last_seen: [Position; 2],
}

impl NeuralNetwork {
    pub fn new(params: Parameters) -> Self {
        Self::new_raw(params, Position::from_str(START_POSITION).unwrap())
    }

    pub fn new_raw(params: Parameters, start_position: Position) -> Self {
        let mut nnue = Self {
            params,
            acc: array::from_fn(|_| vec![0.0; HIDDEN_LAYER_SIZE]),
            last_seen: [start_position; 2],
        };
        for square in Square::list() {
            if let Some((piece, color)) = start_position.piece_color_at(*square) {
                nnue.set(piece, color, *square, Color::White);
                nnue.set(piece, color, *square, Color::Black);
            }
        }
        nnue
    }

    fn perspective_index(perspective: Perspective) -> usize {
        perspective as usize
    }

    fn input_index(piece: Piece, color: Color, square: Square) -> usize {
        (color as usize) * 64 * 6 + (piece as usize) * 64 + square as usize
    }

    fn transformed_square(square: Square, perspective: Perspective) -> Square {
        if perspective == Color::White { square } else { square.flip_vertically() }
    }

    fn transformed_color(color: Color, perspective: Perspective) -> Color {
        if perspective == Color::White { color } else { color.flipped() }
    }

    fn relu(value: f64) -> f64 {
        value.max(0.0)
    }

    fn set(&mut self, piece: Piece, color: Color, square: Square, perspective: Perspective) {
        let input_idx = Self::input_index(
            piece,
            Self::transformed_color(color, perspective),
            Self::transformed_square(square, perspective),
        );
        let perspective_idx = Self::perspective_index(perspective);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[perspective_idx][i] += self.params.acc_weights[i * INPUT_SIZE + input_idx];
        }
    }

    fn clear(&mut self, piece: Piece, color: Color, square: Square, perspective: Perspective) {
        let input_idx = Self::input_index(
            piece,
            Self::transformed_color(color, perspective),
            Self::transformed_square(square, perspective),
        );
        let perspective_idx = Self::perspective_index(perspective);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[perspective_idx][i] -= self.params.acc_weights[i * INPUT_SIZE + input_idx];
        }
    }

    fn forward(&self, perspective: Perspective) -> f64 {
        let mut eval: f64 = 0.0;
        let perspective_idx = Self::perspective_index(perspective);

        for i in 0..HIDDEN_LAYER_SIZE {
            let hidden_out = Self::relu(self.acc[perspective_idx][i] + self.params.acc_biases[i]);
            eval += hidden_out * self.params.out_weights[i];
        }

        eval + self.params.out_bias
    }

    fn forward_and_cache(&mut self, position: &Position) -> f64 {
        let perspective = position.side_to_move();
        let res = self.forward(perspective);
        self.last_seen[Self::perspective_index(perspective)] = *position;
        res
    }

    fn evaluate_unscaled(&mut self, position: &Position) -> f64 {
        let perspective = position.side_to_move();
        let perspective_idx = Self::perspective_index(perspective);
        let diff = position.diff(&self.last_seen[perspective_idx]);
        for e in diff {
            match e {
                PositionDiffEntry::Set(square, piece, color) => self.set(piece, color, square, perspective),
                PositionDiffEntry::Clear(square, piece, color) => self.clear(piece, color, square, perspective),
            }
        }
        self.forward_and_cache(position)
    }

    pub fn evaluate(&mut self, position: &Position) -> ValueScore {
        (self.evaluate_unscaled(position) * SCALE).round() as ValueScore * position.side_to_move().sign() as ValueScore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulator1() {
        // Set all accumulator weights to 1, and biases to 0.
        let params = Parameters::filled(1.0, 0.0, 0.0, 0.0);
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        // Independently of the square, all accumulator nodes will be fed with 1.
        net.set(Piece::Queen, Color::White, Square::E4, Color::White);

        net.acc[0].iter().for_each(|&x| assert_eq!(x, 1.0));
    }

    #[test]
    fn accumulator2() {
        // Set all accumulator weights to 1, except for the White Queen on E4.
        let mut params = Parameters::filled(1.0, 0.0, 0.0, 0.0);

        let queen_e4_index = NeuralNetwork::input_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[i * INPUT_SIZE + queen_e4_index] = 2.0;
        }
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        net.set(Piece::Queen, Color::White, Square::E4, Color::White);
        net.acc[0].iter().for_each(|&x| assert_eq!(x, 2.0));

        net.set(Piece::Rook, Color::White, Square::E4, Color::White);
        net.acc[0].iter().for_each(|&x| assert_eq!(x, 3.0));

        net.clear(Piece::Queen, Color::White, Square::E4, Color::White);
        net.acc[0].iter().for_each(|&x| assert_eq!(x, 1.0));

        net.clear(Piece::Rook, Color::White, Square::E4, Color::White);
        net.acc[0].iter().for_each(|&x| assert_eq!(x, 0.0));
    }

    #[test]
    fn accumulator_perspective() {
        // Set all accumulator weights to 1, except for the Black Queen in E5, mirrored as White in E4.
        let mut params = Parameters::filled(1.0, 0.0, 0.0, 0.0);

        let queen_e5_index = NeuralNetwork::input_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[i * INPUT_SIZE + queen_e5_index] = 2.0;
        }
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        net.set(Piece::Queen, Color::Black, Square::E5, Color::Black);
        net.acc[1].iter().for_each(|&x| assert_eq!(x, 2.0));

        net.set(Piece::Rook, Color::Black, Square::E5, Color::Black);
        net.acc[1].iter().for_each(|&x| assert_eq!(x, 3.0));

        net.clear(Piece::Queen, Color::Black, Square::E5, Color::Black);
        net.acc[1].iter().for_each(|&x| assert_eq!(x, 1.0));

        net.clear(Piece::Rook, Color::Black, Square::E5, Color::Black);
        net.acc[1].iter().for_each(|&x| assert_eq!(x, 0.0));
    }

    #[test]
    fn forward() {
        // Set all accumulator weights to 1, and biases to 0.
        let params = Parameters::filled(1.0, 2.0, 1.0, 10.0);
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        // Set the Queen on E4, which will set all accumulators to 1.
        net.set(Piece::Queen, Color::White, Square::E4, Color::White);
        assert_eq!(net.forward(Color::White), HIDDEN_LAYER_SIZE as f64 * 3.0 + 10.0);

        // Set the Rook on E4, which will add 1 to all accumulators.
        net.set(Piece::Rook, Color::White, Square::E4, Color::White);
        assert_eq!(net.forward(Color::White), HIDDEN_LAYER_SIZE as f64 * 4.0 + 10.0);
    }

    #[test]
    fn forward_perspective() {
        // Set all accumulator weights to 1, and biases to 0.
        let params = Parameters::filled(1.0, 2.0, 1.0, 10.0);
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        // Set a mirrored Knight on F3, which will set all accumulators to 1 for the Black perspective.
        net.set(Piece::Knight, Color::White, Square::F3, Color::Black);
        assert_eq!(net.forward(Color::Black), HIDDEN_LAYER_SIZE as f64 * 3.0 + 10.0);

        // Set a mirrored Bishop on F3, which will add 1 to all accumulators for the Black perspective.
        net.set(Piece::Bishop, Color::White, Square::F3, Color::Black);
        assert_eq!(net.forward(Color::Black), HIDDEN_LAYER_SIZE as f64 * 4.0 + 10.0);
    }

    #[test]
    fn evaluate() {
        // Set all weights to 1, except for the White Queen on E4.
        let mut params = Parameters::filled(1.0, 0.0, 1.0, 0.0);

        let queen_e4_index = NeuralNetwork::input_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[i * INPUT_SIZE + queen_e4_index] = 2.0;
        }
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        let mut position = Position::default();
        position.set_square(Square::E4, Piece::Queen, Color::White);

        assert_eq!(net.evaluate_unscaled(&position), 2.0 * HIDDEN_LAYER_SIZE as f64);

        assert_eq!(net.last_seen[0], position);

        assert_eq!(net.evaluate_unscaled(&position), 2.0 * HIDDEN_LAYER_SIZE as f64);

        position.clear_square(Square::E4);
        assert_eq!(net.evaluate_unscaled(&position), 0.0);

        position.set_square(Square::E4, Piece::Rook, Color::White);
        assert_eq!(net.evaluate_unscaled(&position), HIDDEN_LAYER_SIZE as f64);
    }

    #[test]
    fn evaluate_perspective() {
        // Set all accumulator weights to 1, except for the Black Queen in E5, mirrored as White in E4.
        let mut params = Parameters::filled(1.0, 0.0, 1.0, 0.0);

        let queen_e5_index = NeuralNetwork::input_index(Piece::Queen, Color::White, Square::E4);
        for i in 0..HIDDEN_LAYER_SIZE {
            params.acc_weights[i * INPUT_SIZE + queen_e5_index] = 2.0;
        }
        let mut net = NeuralNetwork::new_raw(params, Position::default());

        let mut position = Position::default().make_null_move();
        position.set_square(Square::E5, Piece::Queen, Color::Black);

        assert_eq!(net.evaluate_unscaled(&position), 2.0 * HIDDEN_LAYER_SIZE as f64);

        assert_eq!(net.last_seen[1], position);

        assert_eq!(net.evaluate_unscaled(&position), 2.0 * HIDDEN_LAYER_SIZE as f64);

        position.clear_square(Square::E5);
        assert_eq!(net.evaluate_unscaled(&position), 0.0);

        position.set_square(Square::E5, Piece::Rook, Color::Black);
        assert_eq!(net.evaluate_unscaled(&position), HIDDEN_LAYER_SIZE as f64);
    }
}
