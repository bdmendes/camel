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
    search::{
        SearchStatus,
        pvs::score_table::{DEFAULT_TABLE_SIZE_MB, ScoreTable},
    },
};

pub mod repl;

pub struct Engine {
    pub position: Position,
    pub evaluator: Arc<Mutex<NeuralNetwork>>,
    pub score_table: Arc<Mutex<ScoreTable>>,
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
            score_table: {
                let table = ScoreTable::new(DEFAULT_TABLE_SIZE_MB);
                Arc::new(Mutex::new(table))
            },
            search_status: SearchStatus::default(),
        }
    }
}
