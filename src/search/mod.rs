mod pvs;

use std::collections::HashMap;

use crate::{
    evaluate::{piece_value, psqt::psqt_value, Evaluation, Score},
    position::{
        moves::{position_is_check, Move},
        zobrist::ZobristHash,
        Piece, Position,
    },
};

use self::pvs::pvsearch;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoundEvaluation {
    pub evaluation: Evaluation,
    pub bound: Bound,
}

impl BoundEvaluation {
    pub fn new(evaluation: Evaluation, bound: Bound) -> BoundEvaluation {
        BoundEvaluation { evaluation, bound }
    }
}

impl std::fmt::Display for BoundEvaluation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.bound {
            Bound::Exact => write!(f, "{}", self.evaluation),
            Bound::Lower => write!(f, "≥{}", self.evaluation),
            Bound::Upper => write!(f, "≤{}", self.evaluation),
        }
    }
}

#[allow(dead_code)]
pub struct Searcher {
    pub move_value_table: HashMap<(Move, ZobristHash), Score>,
    pub position_value_table: HashMap<ZobristHash, (BoundEvaluation, u8)>,
}

#[allow(dead_code)]
impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            move_value_table: HashMap::new(),
            position_value_table: HashMap::new(),
        }
    }

    fn is_quiet_move(move_: &Move, position: &Position) -> bool {
        let promotion = move_.promotion.is_some();
        let capture_piece = move_.capture
            && match position.at(move_.to).unwrap() {
                Piece::WP | Piece::BP => false,
                _ => true,
            };
        !promotion && !capture_piece
    }

    fn move_heuristic_value(move_: &Move, position: &Position) -> Score {
        let mut score: Score = 0;

        if move_.promotion.is_some() {
            score += 3 * piece_value(move_.promotion.unwrap()); // usually ~2700 if queen
        }

        let moved_piece = position.at(move_.from).unwrap();

        if move_.capture {
            let moved_piece_value = piece_value(moved_piece);
            let captured_piece_value = piece_value(position.at(move_.to).unwrap());
            let value_diff = captured_piece_value - moved_piece_value; // if negative, we're losing material
            score += value_diff + piece_value(Piece::WQ); // [~100, ~2000]; equal trade is ~1000
        }

        let start_psqt_value = psqt_value(moved_piece, move_.from, 0);
        let end_psqt_value = psqt_value(moved_piece, move_.to, 0);
        let psqt_value_diff = end_psqt_value - start_psqt_value;
        score += psqt_value_diff as Score + 200; // [~0, ~400]

        score
    }

    fn game_over_evaluation(position: &Position, moves: &Vec<Move>) -> Option<Evaluation> {
        // Flag 50 move rule draws
        if position.half_move_number >= 100 {
            return Some(0.0);
        }

        // Stalemate and checkmate detection
        if moves.len() == 0 {
            let is_check = position_is_check(position, position.to_move, None);
            return match is_check {
                true => Some(f32::MIN),
                false => Some(0.0),
            };
        }

        None
    }

    fn ids(
        &mut self,
        position: &Position,
        max_depth: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        for i in 1..=max_depth {
            let (eval_moves, eval) = pvsearch(self, position, i, f32::MIN, f32::MAX, i, 10);
            if i == max_depth {
                return (eval_moves.unwrap_or(Vec::new()), eval);
            }
        }
        unreachable!()
    }

    pub fn search(
        &mut self,
        position: &Position,
        depth: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        self.ids(position, depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_xray_check() {
        let mut searcher = Searcher::new();
        let position = Position::from_fen("7R/7p/8/3pR1pk/pr1P4/5P2/P6r/3K4 w - - 0 35").unwrap();

        let depth = 2; // quiet search should increase depth due to capture on leaf node
        let (moves, _) = searcher.ids(&position, depth);

        assert_eq!(moves[0].0.to_string(), "h8h7");
    }

    #[test]
    fn search_double_attack() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("2kr3r/ppp2q2/4p2p/3nn3/2P3p1/1B5Q/P1P2PPP/R1B1K2R w KQ - 0 17")
                .unwrap();

        let depth = 2; // quiet search should increase depth due to capture on leaf node
        let (moves, _) = searcher.ids(&position, depth);

        assert_eq!(moves[0].0.to_string(), "h3g3");
    }

    #[test]
    fn search_mate_pattern() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("q5k1/3R2pp/p3pp2/N1b5/4b3/2B2r2/6PP/4QB1K b - - 5 35").unwrap();

        let depth = 5; // needed to find forcing combination

        //let (moves, _) = searcher.iterative_deepening_search(&position, depth);
        //assert_eq!(moves[0].0.to_string(), "f3f2");
    }
}
