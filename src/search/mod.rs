use std::collections::HashMap;

use crate::{
    evaluate::{evaluate_position, piece_value, Evaluation, Score},
    position::{
        movegen::{legal_moves, make_move, position_is_check, Move},
        zobrist::ZobristHash,
        Color, Piece, Position,
    },
};

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

pub struct TranspositionTableValue {
    pub depth: u8,
    pub evaluation: BoundEvaluation,
    pub best_moves: Vec<(Move, BoundEvaluation)>,
}

#[allow(dead_code)]
pub struct Searcher {
    transposition_table: HashMap<ZobristHash, TranspositionTableValue>,
}

#[allow(dead_code)]
impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            transposition_table: HashMap::new(),
        }
    }

    fn is_quiet_move(move_: &Move) -> bool {
        !move_.capture && move_.promotion.is_none()
    }

    fn move_heuristic_value(move_: &Move, position: &Position) -> Score {
        let mut score: Score = 0;

        if move_.promotion.is_some() {
            score += 3 * piece_value(move_.promotion.unwrap()); // usually ~2700 if queen
        }

        if move_.capture {
            let moved_piece_value = piece_value(position.at(move_.from).unwrap());
            let captured_piece_value = piece_value(position.at(move_.to).unwrap());
            let value_diff = captured_piece_value - moved_piece_value; // if negative, we're losing material
            score += value_diff + piece_value(Piece::Queen(Color::White)); // [~100, ~2000]; equal trade is ~1000
        }

        if move_.check {
            score += 2000;
        }

        score
    }

    fn negamax(
        &mut self,
        position: &Position,
        depth: u8,
        alpha: Evaluation,
        beta: Evaluation,
        original_depth: u8,
        quiet_search_depth: u8,
    ) -> (Option<Vec<(Move, BoundEvaluation)>>, BoundEvaluation) {
        // Flag 50 move rule draws
        if position.half_move_number >= 100 {
            return (Some(Vec::new()), BoundEvaluation::new(0.0, Bound::Exact));
        }

        // Check transposition table
        let zobrist_hash = position.to_zobrist_hash();
        let memo = self.transposition_table.get(&zobrist_hash);
        let mut moves = Vec::new();
        let mut override_table = true;
        if let Some(memo) = memo {
            if memo.depth >= depth {
                return (Some(memo.best_moves.clone()), memo.evaluation);
            }
            moves = memo.best_moves.iter().map(|(m, _)| m.clone()).collect();
            override_table = false;
        } else {
            moves = legal_moves(position, position.to_move);
        }

        // Stalemate and checkmate detection
        if moves.len() == 0 {
            let is_check = position_is_check(position, position.to_move, None);
            return (
                Some(Vec::new()),
                match is_check {
                    true => BoundEvaluation::new(f32::MIN, Bound::Exact),
                    false => BoundEvaluation::new(0.0, Bound::Exact),
                },
            );
        }

        let mut quiet_search = false;
        if depth == 0 {
            // Switch to quiet search if there are non-quiet moves
            if quiet_search_depth > 0 {
                if moves.iter().any(|m| !Self::is_quiet_move(m)) {
                    quiet_search = true;
                }
            }

            // No depth left at quiet position, evaluate at leaf node
            if !quiet_search {
                let position_evaluation = evaluate_position(position);
                let score = match position.to_move {
                    Color::White => position_evaluation,
                    Color::Black => -position_evaluation,
                };
                let bound = BoundEvaluation::new(score, Bound::Exact);
                self.transposition_table.insert(
                    zobrist_hash,
                    TranspositionTableValue {
                        depth,
                        evaluation: bound,
                        best_moves: Vec::new(),
                    },
                );
                return (None, bound);
            }
        }

        // Order moves by heuristic value
        moves.sort_by(|m1, m2| {
            Self::move_heuristic_value(m2, position)
                .partial_cmp(&Self::move_heuristic_value(m1, position))
                .unwrap()
        });

        // Traverse the move tree
        let mut eval_moves = Vec::new();
        let mut best_score = f32::MIN;
        let mut new_alpha = alpha;
        for move_ in moves {
            if quiet_search && Self::is_quiet_move(&move_) {
                continue;
            }

            let new_position = make_move(position, &move_);
            let new_depth = if quiet_search { 0 } else { depth - 1 };
            let evaluation = self
                .negamax(
                    &new_position,
                    new_depth,
                    -beta,
                    -new_alpha,
                    original_depth,
                    if quiet_search {
                        quiet_search_depth - 1
                    } else {
                        quiet_search_depth
                    },
                )
                .1;
            let score = -evaluation.evaluation;

            if score > best_score {
                best_score = score;
            }

            if score > new_alpha {
                new_alpha = score;
            }

            // Alpha-beta pruning; fail low
            if new_alpha >= beta {
                return (None, BoundEvaluation::new(new_alpha, Bound::Upper));
            }

            if depth == original_depth {
                eval_moves.push((move_, BoundEvaluation::new(score, evaluation.bound)));
            }
        }

        // Sort moves by score
        eval_moves.sort_by(|a, b| b.1.evaluation.partial_cmp(&a.1.evaluation).unwrap());

        // Store in transposition table if higher depth
        if override_table {
            self.transposition_table.insert(
                zobrist_hash,
                TranspositionTableValue {
                    depth,
                    evaluation: BoundEvaluation::new(best_score, Bound::Exact),
                    best_moves: eval_moves.clone(),
                },
            );
        }

        (
            Some(eval_moves),
            BoundEvaluation::new(new_alpha, Bound::Exact),
        )
    }

    pub fn search(
        &mut self,
        position: &Position,
        depth: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        let (eval_moves, eval) = self.negamax(position, depth, f32::MIN, f32::MAX, depth, 10);
        (eval_moves.unwrap(), eval)
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
        let (moves, _) = searcher.search(&position, depth);

        assert_eq!(moves[0].0.to_string(), "h8h7");
    }

    #[test]
    fn search_double_attack() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("2kr3r/ppp2q2/4p2p/3nn3/2P3p1/1B5Q/P1P2PPP/R1B1K2R w KQ - 0 17")
                .unwrap();

        let depth = 4; // needed to find forcing combination
        let (moves, _) = searcher.search(&position, 4);

        assert_eq!(moves[0].0.to_string(), "h3g3");
    }

    #[test]
    fn search_mate_pattern() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("q5k1/3R2pp/p3pp2/N1b5/4b3/2B2r2/6PP/4QB1K b - - 5 35").unwrap();

        let depth = 5; // needed to find forcing combination
        let (moves, _) = searcher.search(&position, 5);

        assert_eq!(moves[0].0.to_string(), "f3f2");
    }
}
