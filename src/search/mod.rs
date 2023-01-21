use std::collections::HashMap;

use crate::{
    evaluate::{evaluate_position, Evaluation},
    position::{
        moves::{legal_moves, make_move, Move},
        zobrist::ZobristHash,
        Color, Position,
    },
};

struct Searcher {
    eval_transposition_table: HashMap<ZobristHash, Evaluation>,
}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {
            eval_transposition_table: HashMap::new(),
        }
    }

    pub fn search(
        &mut self,
        position: &Position,
        depth: u8,
        alpha: Evaluation,
        beta: Evaluation,
    ) -> (Vec<(Move, Evaluation)>, Evaluation) {
        // negamax; wip

        if depth == 0 {
            let evaluation = evaluate_position(position);
            return (
                Vec::new(),
                match position.to_move {
                    Color::White => evaluation,
                    Color::Black => -evaluation,
                },
            );
        }

        let mut res = Vec::new();
        let moves = legal_moves(position);
        let mut best_score = -999999.0;

        // moves.sort_by(|a, b| {
        //     let a_score = self.evaluate_move(position, a);
        //     let b_score = self.evaluate_move(position, b);
        //     b_score.partial_cmp(&a_score).unwrap()
        // });

        for move_ in moves {
            let new_position = make_move(position, &move_);
            let score = -self.search(&new_position, depth - 1, -beta, -alpha).1;
            res.push((move_, score));
            if score > best_score {
                best_score = score;
            }
        }

        res.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        (res, best_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_initial_position() {
        let mut searcher = Searcher::new();
        let position = Position::new();
        let (moves, score) = searcher.search(&position, 3, -999999.0, 999999.0);
        assert_eq!(moves.len(), 20);
        for (move_, score) in moves {
            println!("{} {}", move_, score);
        }
    }

    #[test]
    fn search_test() {
        let mut searcher = Searcher::new();
        let position = Position::from_fen("7R/7p/8/3pR1pk/pr1P4/5P2/P6r/3K4 w - - 0 35").unwrap();
        let (moves, score) = searcher.search(&position, 4, -999999.0, 999999.0);
        for (move_, score) in moves {
            println!("{} {}", move_, score);
        }
    }
}
