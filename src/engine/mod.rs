use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    evaluation::{
        MAX_POSITIONAL_WEIGHT, NNUE_PARAMS_BLOB,
        nnue::{NeuralNetwork, Parameters},
        score::Score,
    },
    position::{MoveStage, Position, fen::START_POSITION},
    search::{
        Depth, MAX_DEPTH, Searcher,
        game_history::GameHistory,
        score_table::{DEFAULT_TABLE_SIZE_MB, ScoreTable},
        status::{SearchStatus, SearchStatusValue},
        window::Window,
    },
};

pub mod repl;
pub mod time;

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
    pub fn go(&self, depth: Option<Depth>, duration: Option<Duration>) {
        let history = self.game_history.clone();
        let table = self.score_table.clone();
        let net = self.evaluator.clone();
        let status = self.search_status.clone();
        let position = self.position;

        thread::spawn(move || {
            let mut history = history.lock().unwrap();
            let mut table = table.lock().unwrap();
            let mut net = net.lock().unwrap();
            let duration = duration.unwrap_or(Duration::from_hours(1));
            let available_moves = position.moves(MoveStage::All).len();

            table.prepare_new_search();

            let mut searcher = Searcher::new(&mut history, &mut table, &mut net, status.clone(), duration);
            let mut pv = Vec::new();

            for d in 1..=depth.unwrap_or(MAX_DEPTH) {
                let time = Instant::now();
                let (nodes, score) = searcher.pvs(&position, d, 0, Window::default());

                if d > 1 && searcher.should_stop() {
                    break;
                }

                let elapsed = time.elapsed();
                let score = Score::from(score);
                pv = searcher.pv(&position);
                let is_mate = matches!(score, Score::Mate(_));
                if !is_mate {
                    pv.truncate(d as usize);
                }
                println!(
                    "info depth {} score {} time {} nodes {} nps {} hashfull {} pv {}",
                    d,
                    score,
                    elapsed.as_millis(),
                    nodes,
                    (nodes as f64 / elapsed.as_secs_f64()) as u64,
                    searcher.hashfull_millis(),
                    pv.join(" ")
                );

                if pv.is_empty()
                    || elapsed > duration / 2
                    || is_mate
                    || (!matches!(score, Score::Value(v) if v.abs() > MAX_POSITIONAL_WEIGHT) && available_moves <= 1)
                {
                    break;
                }
            }

            status.set(SearchStatusValue::Stopped);

            if let Some(best_move) = pv.first() {
                if let Some(ponder_move) = pv.get(1) {
                    println!("bestmove {} ponder {}", best_move, ponder_move);
                } else {
                    println!("bestmove {}", best_move);
                }
            } else {
                println!("bestmove (none)");
            }
        });
    }
}
