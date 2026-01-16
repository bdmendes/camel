use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    evaluation::{
        NNUE_PARAMS_BLOB, ValueScore,
        nnue::{NeuralNetwork, Parameters},
    },
    position::{Position, fen::START_POSITION},
    search::{
        NodeType, Searcher,
        game_history::GameHistory,
        score_table::{DEFAULT_TABLE_SIZE_MB, ScoreTable},
        status::{SearchStatus, SearchStatusValue},
        window::Window,
    },
};

pub mod repl;

pub struct Engine {
    pub position: Position,
    pub evaluator: Arc<Mutex<NeuralNetwork>>,
    pub score_table: Arc<Mutex<ScoreTable>>,
    pub game_history: Arc<Mutex<GameHistory>>,
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
            game_history: Arc::new(Mutex::new(GameHistory::default())),
            search_status: SearchStatus::default(),
        }
    }
}

impl Engine {
    pub fn go(&self) {
        let history = self.game_history.clone();
        let table = self.score_table.clone();
        let net = self.evaluator.clone();
        let status = self.search_status.clone();
        let position = self.position;

        std::thread::spawn(move || {
            let mut history = history.lock().unwrap();
            let mut table = table.lock().unwrap();
            let mut net = net.lock().unwrap();
            let duration = Duration::from_hours(1);

            let mut searcher = Searcher::new(&mut history, &mut table, &mut net, status.clone(), duration);

            status.set(SearchStatusValue::Searching);
            searcher.pvs(&position, 5, 0, NodeType::PVNode, Window::default());
            status.set(SearchStatusValue::Stopped);
            println!("move: {}", table.hash_move(&position).unwrap());
            println!(
                "eval: {}",
                table.probe(&position, 5, 0).unwrap().0 * position.side_to_move().sign() as ValueScore
            );
        });
    }
}
