use std::{fs::File, io::BufRead, str::FromStr};

use crate::{core::Position, evaluation::nnue::SCALE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Result {
    Win,
    Draw,
    Loss,
}

impl Result {
    pub fn to_score(self) -> i16 {
        match self {
            Result::Win => -SCALE,
            Result::Draw => 0,
            Result::Loss => SCALE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionScore {
    pub position: Position,
    pub result: Result,
}

pub fn load_scored_epd(path: &str) -> Vec<PositionScore> {
    let mut dataset = Vec::new();
    let file = File::open(path).expect("Failed to open dataset file");

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line from dataset file");

        let parts = line.split_whitespace();
        let fen = parts.clone().take(4).collect::<Vec<&str>>().join(" ");
        let res = parts.clone().nth(5).unwrap();

        let position = Position::from_str(&fen).expect("Invalid FEN string");
        let result = match res.trim() {
            "\"1-0\";" => Result::Win,
            "\"0-1\";" => Result::Loss,
            "\"1/2-1/2\";" => Result::Draw,
            _ => panic!("Invalid result format in dataset file: {}", res),
        };

        dataset.push(PositionScore { position, result });
    }

    dataset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_quiet_labeled() {
        let dataset = load_scored_epd("assets/books/quiet-labeled.epd");

        assert_eq!(dataset.len(), 1_428_000);

        let first = dataset.first().unwrap();
        assert_eq!(
            first.position.fen(),
            "r2qkr2/p1pp1ppp/1pn1pn2/2P5/3Pb3/2N1P3/PP3PPP/R1B1KB1R b KQq - 0 1"
        );
        assert_eq!(first.result, Result::Loss);

        let last = dataset.last().unwrap();
        assert_eq!(last.position.fen(), "4r3/8/8/5p2/5k2/8/K7/6n1 b - - 0 1");
        assert_eq!(last.result, Result::Loss);
    }
}
