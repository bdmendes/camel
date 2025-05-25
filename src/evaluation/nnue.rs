use crate::core::{color::Color, piece::Piece, square::Square, Position};
use rand::Rng;

// 2 sides, 6 pieces, 64 squares.
const INPUT_SIZE: usize = 768;

// We have a single hidden layer in our network.
const HIDDEN_LAYER_SIZE: usize = 128;

// This is only relevant to scale the training data:
// 0-1 to -SCALE, 0.5-0.5 to 0, and 1-0 to +SCALE.
// This is a rough mapping to centipawns, and a way
// to deal with the fact that integers are easier.
pub const SCALE: i16 = 400;

pub struct Parameters {
    // The "accumulator" is the cached input of the hidden layer.
    // In practice, it will be 0 (empty) or 1 (set) times the weights.
    pub acc_weights: [i32; INPUT_SIZE * HIDDEN_LAYER_SIZE],
    pub acc_biases: [i32; HIDDEN_LAYER_SIZE],

    // The output of the hidden layer is fed to the "output"
    // parameters to generate the final static evaluation.
    pub out_weights: [i32; HIDDEN_LAYER_SIZE],
    pub out_bias: i16,
}

impl Parameters {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        let acc_weights = [0; INPUT_SIZE * HIDDEN_LAYER_SIZE].map(|_| rng.gen_range(-128..=127));
        let acc_biases = [0; HIDDEN_LAYER_SIZE].map(|_| rng.gen_range(-128..=127));
        let out_weights = [0; HIDDEN_LAYER_SIZE].map(|_| rng.gen_range(-128..=127));
        let out_bias = rng.gen_range(-128..=127);

        Self { acc_weights, acc_biases, out_weights, out_bias }
    }
}

pub struct NeuralNetwork {
    acc: [i32; HIDDEN_LAYER_SIZE],
    params: Parameters,
    last_seen_position: Option<Position>,
    last_result: i16,
}

impl NeuralNetwork {
    fn new(params: Parameters) -> Self {
        Self { acc: [0; HIDDEN_LAYER_SIZE], params, last_seen_position: None, last_result: 0 }
    }

    fn acc_index(piece: Piece, color: Color, square: Square) -> usize {
        (color as usize) * 64 * 6 + (piece as usize) * 64 + square as usize
    }

    fn set(&mut self, piece: Piece, color: Color, square: Square) {
        let idx = Self::acc_index(piece, color, square);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[i] += self.params.acc_weights[idx * HIDDEN_LAYER_SIZE + i];
        }
    }

    fn unset(&mut self, piece: Piece, color: Color, square: Square) {
        let idx = Self::acc_index(piece, color, square);
        for i in 0..HIDDEN_LAYER_SIZE {
            self.acc[i] -= self.params.acc_weights[idx * HIDDEN_LAYER_SIZE + i];
        }
    }

    fn forward(&self) -> i16 {
        let mut eval: i32 = 0;

        for i in 0..HIDDEN_LAYER_SIZE {
            // Activate with a clipped ReLU, with bounds 0 and 255.
            let hidden_out = (self.acc[i] + self.params.acc_biases[i]).clamp(0, 255);
            eval += hidden_out.saturating_mul(self.params.out_weights[i]);
        }

        eval as i16 + self.params.out_bias
    }

    pub fn evaluate(&mut self, position: &Position) -> i16 {
        if Some(position.hash()) == self.last_seen_position.map(|p| p.hash()) {
            self.last_result
        } else {
            // TODO: diff with position

            self.last_seen_position = Some(*position);
            let res = self.forward();
            self.last_result = res;
            res
        }
    }
}
