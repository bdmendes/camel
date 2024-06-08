// This is a very small tuner, tuning only 5 parameters: the piece values.
// PSQT are purposely not tuned to speed up the process.
// The next Camel major version will switch to NNUE, which won't require
// Texel tuning anymore, so this is a small, temporary solution.

use std::ptr::addr_of_mut;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    evaluation::{
        self,
        position::{
            bishops::BISHOP_PAIR_BONUS,
            king::SHELTER_PENALTY,
            pawns::{DOUBLED_PAWNS_PENALTY, PAWN_ISLAND_PENALTY},
            rooks::{OPEN_FILE_BONUS, SEMI_OPEN_FILE_BONUS},
        },
        ValueScore,
    },
    position::{fen::FromFen, Color, Position},
    search::{constraint::SearchConstraint, quiesce::quiesce},
};

const NUMBER_PARAMETERS: usize = 16;
const NUMBER_POSITIONS: usize = 300000;

struct PositionEntry {
    winner: Option<Color>,
    position: Position,
}

impl PositionEntry {
    fn new(winner: Option<Color>, position: Position) -> Self {
        Self { winner, position }
    }

    fn score(&self) -> f64 {
        match self.winner {
            Some(Color::White) => 1.0,
            Some(Color::Black) => 0.0,
            None => 0.5,
        }
    }
}

unsafe fn get_parameter(idx: usize) -> *mut ValueScore {
    match idx {
        0 => addr_of_mut!(evaluation::PAWN_VALUE),
        1 => addr_of_mut!(evaluation::KNIGHT_VALUE),
        2 => addr_of_mut!(evaluation::BISHOP_VALUE),
        3 => addr_of_mut!(evaluation::ROOK_VALUE),
        4 => addr_of_mut!(evaluation::QUEEN_VALUE),
        5 => addr_of_mut!(evaluation::position::PAWN_MIDGAME_RATIO),
        6 => addr_of_mut!(evaluation::position::KNIGHT_MIDGAME_RATIO),
        7 => addr_of_mut!(evaluation::position::BISHOP_MIDGAME_RATIO),
        8 => addr_of_mut!(evaluation::position::ROOK_MIDGAME_RATIO),
        9 => addr_of_mut!(evaluation::position::QUEEN_MIDGAME_RATIO),
        10 => addr_of_mut!(BISHOP_PAIR_BONUS),
        11 => addr_of_mut!(SHELTER_PENALTY),
        12 => addr_of_mut!(DOUBLED_PAWNS_PENALTY),
        13 => addr_of_mut!(PAWN_ISLAND_PENALTY),
        14 => addr_of_mut!(SEMI_OPEN_FILE_BONUS),
        15 => addr_of_mut!(OPEN_FILE_BONUS),
        _ => panic!("Invalid parameter index"),
    }
}

unsafe fn set_parameters(parameters: &[ValueScore]) {
    evaluation::PAWN_VALUE = parameters[0];
    evaluation::KNIGHT_VALUE = parameters[1];
    evaluation::BISHOP_VALUE = parameters[2];
    evaluation::ROOK_VALUE = parameters[3];
    evaluation::QUEEN_VALUE = parameters[4];
    evaluation::position::PAWN_MIDGAME_RATIO = parameters[5];
    evaluation::position::KNIGHT_MIDGAME_RATIO = parameters[6];
    evaluation::position::BISHOP_MIDGAME_RATIO = parameters[7];
    evaluation::position::ROOK_MIDGAME_RATIO = parameters[8];
    evaluation::position::QUEEN_MIDGAME_RATIO = parameters[9];
    BISHOP_PAIR_BONUS = parameters[10];
    SHELTER_PENALTY = parameters[11];
    DOUBLED_PAWNS_PENALTY = parameters[12];
    PAWN_ISLAND_PENALTY = parameters[13];
    SEMI_OPEN_FILE_BONUS = parameters[14];
    OPEN_FILE_BONUS = parameters[15];
}

fn evaluation_error(entries: &[PositionEntry], k: f64) -> f64 {
    let sigmoid = |x: f64| 1.0 / (1.0 + (10.0_f64).powf(-k * x / 400.0));
    let error = entries
        .par_iter()
        .map(|entry| {
            let score = entry.score();
            let evaluation = quiesce(
                &entry.position,
                ValueScore::MIN + 1,
                ValueScore::MAX,
                &SearchConstraint::default(),
                0,
            )
            .0 * match entry.position.side_to_move {
                Color::White => 1,
                Color::Black => -1,
            };
            (score - sigmoid(evaluation as f64)).powi(2)
        })
        .sum::<f64>();
    error / entries.len() as f64
}

pub fn texel_tune() -> Vec<ValueScore> {
    let entries: Vec<PositionEntry> = {
        let epd_file =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/books/quiet-labeled.epd"));
        epd_file
            .lines()
            .take(NUMBER_POSITIONS)
            .collect::<Vec<&str>>()
            .par_iter()
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let fen = parts.iter().take(4).cloned().collect::<Vec<&str>>().join(" ");
                let winner = match parts[5] {
                    "\"0-1\";" => Some(Color::Black),
                    "\"1-0\";" => Some(Color::White),
                    _ => None,
                };
                let position = Position::from_fen(&fen).unwrap();
                PositionEntry::new(winner, position)
            })
            .collect()
    };

    // Find k that minimizes the error.
    let mut k = 0.5;
    let mut best_error = f64::MAX;
    let mut best_k = k;
    while k < 2.0 {
        let error = evaluation_error(&entries, k);
        if error < best_error {
            best_error = error;
            best_k = k;
        }
        println!("error: {:.4} k: {:.2}", error, k);
        k += 0.01;
    }
    println!("Best k: {:.2}", best_k);
    println!("Best error: {:.4}", best_error);

    let mut improved = true;
    let mut best_error = evaluation_error(&entries, k);
    let mut best_parameters = unsafe {
        (0..NUMBER_PARAMETERS).map(|idx| *get_parameter(idx)).collect::<Vec<ValueScore>>()
    };

    unsafe {
        while improved {
            improved = false;

            for idx in 0..NUMBER_PARAMETERS {
                let mut parameters = best_parameters.clone();

                if parameters[idx] < ValueScore::MAX {
                    parameters[idx] += 1;
                    set_parameters(&parameters);
                    let error = evaluation_error(&entries, best_k);
                    if error < best_error {
                        best_error = error;
                        improved = true;
                        best_parameters = parameters;
                    } else if parameters[idx] > 2 {
                        parameters[idx] -= 2;
                        set_parameters(&parameters);
                        let error = evaluation_error(&entries, best_k);
                        if error < best_error {
                            best_error = error;
                            improved = true;
                            best_parameters = parameters;
                        }
                    }
                }

                set_parameters(&best_parameters);
            }

            print!("current values: ");
            for idx in 0..NUMBER_PARAMETERS {
                print!("{} ", *get_parameter(idx));
            }
            println!("; error {:.8}", best_error);
        }
    }

    println!("Best error: {:.4}", best_error);
    unsafe { (0..NUMBER_PARAMETERS).map(|idx| *get_parameter(idx)).collect() }
}
