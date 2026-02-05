use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    evaluation::{
        NNUE_PARAMS_BLOB,
        nnue::{NeuralNetwork, Parameters},
    },
    position::{Position, fen::START_POSITION},
    search::{
        Depth, Searcher,
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
        let start_position = Position::from_str(START_POSITION).unwrap();
        Self {
            position: start_position,
            evaluator: {
                let params = Parameters::from_str(NNUE_PARAMS_BLOB).unwrap();
                Arc::new(Mutex::new(NeuralNetwork::new(params)))
            },
            score_table: {
                let table = ScoreTable::new(DEFAULT_TABLE_SIZE_MB);
                Arc::new(Mutex::new(table))
            },
            game_history: Arc::new(Mutex::new(GameHistory::new(&start_position))),
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

        status.set(SearchStatusValue::Searching);

        thread::spawn(move || {
            let mut history = history.lock().unwrap();
            let mut table = table.lock().unwrap();
            let mut net = net.lock().unwrap();
            let duration = Duration::from_hours(1);

            let mut searcher = Searcher::new(&mut history, &mut table, &mut net, status.clone(), duration);

            for d in 1..=Depth::MAX {
                let time = Instant::now();
                let (nodes, score) = searcher.alphabeta(&position, d, 0, Window::default());
                if searcher.should_stop() {
                    break;
                }
                println!(
                    "info depth {} score cp {} time {} nodes {} nps {} pv {}",
                    d,
                    score,
                    time.elapsed().as_millis(),
                    nodes,
                    (nodes as f64 / time.elapsed().as_secs_f64()) as u64,
                    searcher.pv(&position).join(" ")
                );
            }

            status.set(SearchStatusValue::Stopped);
            println!("bestmove {}", table.hash_move(&position).unwrap());
        });
    }
}
