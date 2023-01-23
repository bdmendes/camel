use std::collections::HashMap;

use crate::{
    evaluate::{evaluate_position, piece_value, psqt::psqt_value, Evaluation, Score},
    position::{
        moves::{legal_moves, make_move, position_is_check, Move},
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

    fn principal_variation_search(
        &mut self,
        position: &Position,
        depth: u8,
        alpha: Evaluation,
        beta: Evaluation,
        original_depth: u8,
        quiet_search_depth: u8,
    ) -> (Option<Vec<(Move, BoundEvaluation)>>, BoundEvaluation) {
        // Check table
        let zobrist_hash = position.to_zobrist_hash();
        if let Some((bound_evaluation, found_depth)) = self.position_value_table.get(&zobrist_hash)
        {
            if *found_depth >= depth {
                //println!("Found position in table");
                return (None, *bound_evaluation);
            }
        }

        let mut moves = legal_moves(position, position.to_move);

        // Check for game over
        if let Some(evaluation) = Self::game_over_evaluation(position, &moves) {
            let bound = BoundEvaluation::new(evaluation, Bound::Exact);
            return (None, bound);
        }

        let mut quiet_search = false;
        if depth == 0 {
            // Switch to quiet search if there are non-quiet moves
            if quiet_search_depth > 0 {
                if moves.iter().any(|m| !Self::is_quiet_move(m, position)) {
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
                return (None, bound);
            }
        }

        // Order moves by heuristic/table value
        moves.sort_by(|m1, m2| {
            let m2_table_value = self.move_value_table.get(&(m2.clone(), zobrist_hash));
            let m1_table_value = self.move_value_table.get(&(m1.clone(), zobrist_hash));
            m2_table_value
                .unwrap_or(&Self::move_heuristic_value(m2, position))
                .cmp(m1_table_value.unwrap_or(&Self::move_heuristic_value(m1, position)))
        });

        if depth == original_depth {
            println!("First move: {}", moves[0]);
        }

        // Traverse the move tree
        let mut eval_moves = Vec::new();
        let mut best_score = f32::MIN;
        let mut new_alpha = alpha;
        let mut before_principal = true;
        for move_ in moves {
            // Skip quiet moves if we're in quiet search
            if quiet_search && Self::is_quiet_move(&move_, position) {
                continue;
            }

            // Search node
            let new_position = make_move(position, &move_);
            let eval = self
                .principal_variation_search(
                    &new_position,
                    if quiet_search || depth == 0 {
                        0
                    } else {
                        depth - 1
                    },
                    if before_principal || quiet_search {
                        -beta
                    } else {
                        -new_alpha - 1.0
                    }, // TODO: debug principal variation search
                    -new_alpha,
                    original_depth,
                    if quiet_search && quiet_search_depth > 0 {
                        quiet_search_depth - 1
                    } else {
                        quiet_search_depth
                    },
                )
                .1;
            let score = -eval.evaluation;

            // If we're at the original depth, store the move
            if depth == original_depth {
                eval_moves.push((move_.clone(), BoundEvaluation::new(score, eval.bound)));
            }

            // Update best score
            if score > best_score {
                best_score = score;
            }
            if score > new_alpha {
                new_alpha = score;
            }

            // Update move value table
            self.move_value_table
                .insert((move_, zobrist_hash), (score * 100.0) as Score);

            // Alpha-beta pruning; fail soft
            if new_alpha >= beta {
                return (None, BoundEvaluation::new(new_alpha, Bound::Upper));
            }

            before_principal = false;
        }

        // Sort moves by score
        eval_moves.sort_by(|a, b| b.1.evaluation.partial_cmp(&a.1.evaluation).unwrap());

        // Update position value table
        let bound = if best_score <= alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        self.position_value_table.insert(
            zobrist_hash,
            (BoundEvaluation::new(best_score, bound), depth),
        );

        (
            Some(eval_moves),
            BoundEvaluation::new(new_alpha, Bound::Exact),
        )
    }

    fn iterative_deepening_search(
        &mut self,
        position: &Position,
        max_depth: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        for i in 1..=max_depth {
            let (eval_moves, eval) =
                self.principal_variation_search(position, i, f32::MIN, f32::MAX, i, 10);
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
        self.iterative_deepening_search(position, depth)
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
        let (moves, _) = searcher.iterative_deepening_search(&position, depth);

        assert_eq!(moves[0].0.to_string(), "h8h7");
    }

    #[test]
    fn search_double_attack() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("2kr3r/ppp2q2/4p2p/3nn3/2P3p1/1B5Q/P1P2PPP/R1B1K2R w KQ - 0 17")
                .unwrap();

        let depth = 2; // quiet search should increase depth due to capture on leaf node
        let (moves, _) = searcher.iterative_deepening_search(&position, depth);

        assert_eq!(moves[0].0.to_string(), "h3g3");
    }

    #[test]
    fn search_mate_pattern() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("q5k1/3R2pp/p3pp2/N1b5/4b3/2B2r2/6PP/4QB1K b - - 5 35").unwrap();

        let depth = 5; // needed to find forcing combination

        let (moves, _) = searcher.iterative_deepening_search(&position, depth);
        assert_eq!(moves[0].0.to_string(), "f3f2");
    }
}
