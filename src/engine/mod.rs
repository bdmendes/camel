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
        picker::MovePicker,
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
    pub fn go(&self, depth: Option<Depth>) {
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

            table.prepare_new_search();

            let mut searcher = Searcher::new(&mut history, &mut table, &mut net, status.clone(), duration);

            for d in 1..=depth.unwrap_or(Depth::MAX) {
                let time = Instant::now();
                let (nodes, score) = searcher.pvs(&position, d, 0, Window::default());
                if searcher.should_stop() {
                    break;
                }
                let pv = searcher.pv(&position);
                let elapsed = time.elapsed();
                println!(
                    "info depth {} score cp {} time {} nodes {} nps {} hashfull {} pv {}",
                    d,
                    score,
                    elapsed.as_millis(),
                    nodes,
                    (nodes as f64 / elapsed.as_secs_f64()) as u64,
                    searcher.hashfull_millis(),
                    pv[..(d as usize).min(pv.len())].join(" ")
                );
            }

            status.set(SearchStatusValue::Stopped);

            if let Some(best_move) = table.hash_move(&position).or_else(|| {
                let mut picker = MovePicker::new(&position, false, None, [None, None]);
                picker.next()
            }) {
                println!("bestmove {}", best_move);
            }
        });
    }
}
