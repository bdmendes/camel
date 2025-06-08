use crate::core::Position;
use std::{fs::File, io::BufRead, str::FromStr};

pub fn load_scored_epd(path: &str) -> Vec<(Position, i16)> {
    let mut dataset = Vec::new();
    let file = File::open(path).expect("Failed to open dataset file");

    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line from dataset file");

        let parts = line.split_whitespace();
        let fen = parts.clone().take(6).collect::<Vec<&str>>().join(" ");
        let res = parts.clone().nth(7).unwrap();

        let position = Position::from_str(&fen).expect("Invalid FEN string");
        let score = res.trim_end_matches(';').trim_matches('"').parse::<i16>().unwrap();

        dataset.push((position, score));
    }

    dataset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_quiet_evaluated() {
        let dataset = load_scored_epd("assets/books/quiet-evaluated-camelv1.epd");

        assert_eq!(dataset.len(), 1_428_000);

        let first = dataset.first().unwrap();
        assert_eq!(
            first.0.fen(),
            "r2qkr2/p1pp1ppp/1pn1pn2/2P5/3Pb3/2N1P3/PP3PPP/R1B1KB1R b KQq - 0 1"
        );
        assert_eq!(first.1, -1176);

        let last = dataset.last().unwrap();
        assert_eq!(last.0.fen(), "4r3/8/8/5p2/5k2/8/K7/6n1 b - - 0 1");
        assert_eq!(last.1, -1137);
    }
}
