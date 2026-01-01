use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::{
    core::position::{Position, fen::START_POSITION},
    evaluation::{
        NNUE_PARAMS_BLOB,
        nnue::{NeuralNetwork, Parameters},
    },
    search::SearchStatus,
};

pub mod repl;

pub struct Engine {
    pub position: Position,
    pub evaluator: Arc<Mutex<NeuralNetwork>>,
    pub search_status: SearchStatus,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            position: Position::from_str(START_POSITION).unwrap(),
            evaluator: {
                let params = Parameters::from_str(NNUE_PARAMS_BLOB).unwrap();
                Arc::new(Mutex::new(NeuralNetwork::new(params)))
            },
            search_status: SearchStatus::default(),
        }
    }
}
