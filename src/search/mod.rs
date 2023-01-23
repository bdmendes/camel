use std::collections::HashMap;

use crate::{
    evaluate::{evaluate_position, Evaluation},
    position::{
        movegen::{make_move, Move, MoveGenerator},
        zobrist::ZobristHash,
        Color, Piece, Position,
    },
};

pub enum Bound {
    Exact,
    Lower,
    Upper,
}

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
struct Searcher {
    move_generator: MoveGenerator,
    eval_transposition_table: HashMap<ZobristHash, Evaluation>,
}

#[allow(dead_code)]
impl Searcher {
    pub fn new(move_generator: MoveGenerator) -> Searcher {
        Searcher {
            eval_transposition_table: HashMap::new(),
            move_generator,
        }
    }

    fn negamax(
        &mut self,
        position: &Position,
        depth: u8,
        alpha: Evaluation,
        beta: Evaluation,
        original_depth: u8,
        quiet_search_depth: u8,
        late_move_reduction_idx: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        // Flag 50 move rule draws
        if position.half_move_number >= 100 {
            return (Vec::new(), BoundEvaluation::new(0.0, Bound::Exact));
        }

        let mut moves = self.move_generator.legal_moves(position, position.to_move);

        // Stalemate and checkmate detection
        if moves.len() == 0 {
            let is_check = MoveGenerator::position_is_check(position, position.to_move, None);
            return (
                Vec::new(),
                match is_check {
                    true => BoundEvaluation::new(f32::MIN, Bound::Exact),
                    false => BoundEvaluation::new(0.0, Bound::Exact),
                },
            );
        }

        let mut quiet_search = false;
        if depth == 0 {
            // Switch to quiet search if there are non-pawn captures
            if quiet_search_depth > 0 {
                for move_ in &moves {
                    if move_.capture {
                        let captured_piece = match position.at(&move_.to) {
                            None | Some(Piece::Pawn(_)) => false,
                            _ => true,
                        };
                        if captured_piece {
                            quiet_search = true;
                            break;
                        }
                    }
                }
            }

            // No depth left at quiet position, evaluate at leaf node
            if !quiet_search {
                let evaluation = evaluate_position(position);
                return (
                    Vec::new(),
                    match position.to_move {
                        Color::White => BoundEvaluation::new(evaluation, Bound::Exact),
                        Color::Black => BoundEvaluation::new(-evaluation, Bound::Exact),
                    },
                );
            }
        }

        // Order moves by capture
        moves.sort_by(|m1, m2| {
            let m1_score = match position.at(&m1.to) {
                None => 0,
                Some(Piece::Pawn(_)) => 1,
                _ => 2,
            };
            let m2_score = match position.at(&m2.to) {
                None => 0,
                Some(Piece::Pawn(_)) => 1,
                _ => 2,
            };
            m2_score.cmp(&m1_score)
        });

        // Traverse the move tree
        let mut eval_moves = Vec::new();
        let mut best_score = f32::MIN;
        let mut new_alpha = alpha;
        let mut move_idx: u8 = 0;
        for move_ in moves {
            move_idx += 1;

            if quiet_search {
                let captured_piece = match position.at(&move_.to) {
                    None | Some(Piece::Pawn(_)) => false,
                    _ => true,
                };
                if !captured_piece {
                    continue;
                }
            }

            let new_position = make_move(position, &move_);
            let new_depth = if quiet_search {
                0
            } else {
                if move_idx > late_move_reduction_idx && depth > 1 {
                    depth - 2
                } else {
                    depth - 1
                }
            };
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
                    late_move_reduction_idx,
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
                return (Vec::new(), BoundEvaluation::new(new_alpha, Bound::Upper));
            }

            if depth == original_depth {
                eval_moves.push((move_, BoundEvaluation::new(score, evaluation.bound)));
            }
        }

        eval_moves.sort_by(|a, b| b.1.evaluation.partial_cmp(&a.1.evaluation).unwrap());
        (eval_moves, BoundEvaluation::new(best_score, Bound::Exact))
    }

    pub fn search(
        &mut self,
        position: &Position,
        depth: u8,
    ) -> (Vec<(Move, BoundEvaluation)>, BoundEvaluation) {
        self.negamax(position, depth, f32::MIN, f32::MAX, depth, 10, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_initial_position() {
        let mut searcher = Searcher::new(MoveGenerator::new());
        let position = Position::new();
        let (moves, _) = searcher.search(&position, 4);
        assert_eq!(moves.len(), 20);
        for (move_, score) in moves {
            println!("{} {}", move_, score);
        }
    }

    #[test]
    fn search_simple_xray_check() {
        let mut searcher = Searcher::new(MoveGenerator::new());
        let position = Position::from_fen("7R/7p/8/3pR1pk/pr1P4/5P2/P6r/3K4 w - - 0 35").unwrap();
        let (moves, _) = searcher.search(&position, 6);
        for (move_, score) in moves {
            println!("{} {}", move_, score);
        }
    }
}
