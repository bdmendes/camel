use std::str::FromStr;

use crate::{
    core::position::{Position, fen::START_POSITION},
    evaluation::nnue::{NeuralNetwork, Parameters},
    search::SearchStatus,
};

static NNUE_PARAMS_BLOB: &str = include_str!("../assets/models/quiet-labeled-20250610.nnue");

pub struct Engine {
    pub position: Position,
    pub evaluator: NeuralNetwork,
    pub search_status: SearchStatus,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            position: Position::from_str(START_POSITION).unwrap(),
            evaluator: {
                let params = Parameters::from_str(NNUE_PARAMS_BLOB).unwrap();
                NeuralNetwork::new(params)
            },
            search_status: SearchStatus::default(),
        }
    }
}
