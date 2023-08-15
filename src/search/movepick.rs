use smallvec::SmallVec;

use crate::{
    evaluation::ValueScore,
    moves::{Move, MoveVec},
};

type ScoredMoveVec = SmallVec<[(Move, ValueScore); 64]>;

pub struct MovePicker {
    index: usize,
    moves: ScoredMoveVec,
}

impl MovePicker {
    pub fn new<F>(moves: &MoveVec, f: F) -> Self
    where
        F: Fn(Move) -> ValueScore,
    {
        Self { index: 0, moves: Self::decorate_moves_with_score(moves, f) }
    }

    fn decorate_moves_with_score<F>(moves: &MoveVec, f: F) -> ScoredMoveVec
    where
        F: Fn(Move) -> ValueScore,
    {
        let mut scored_moves = ScoredMoveVec::new();
        for mov in moves.iter() {
            scored_moves.push((*mov, f(*mov)));
        }
        scored_moves
    }
}

impl std::iter::Iterator for MovePicker {
    type Item = (Move, ValueScore, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.moves.len() {
            return None;
        }

        let mut best_score = self.moves[self.index].1;

        for i in (self.index + 1)..self.moves.len() {
            if self.moves[i].1 > best_score {
                best_score = self.moves[i].1;
                self.moves.swap(i, self.index);
            }
        }

        self.index += 1;
        Some((self.moves[self.index - 1].0, self.moves[self.index - 1].1, self.index - 1))
    }
}
