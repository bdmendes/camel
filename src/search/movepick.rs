use crate::{evaluation::ValueScore, moves::Move};

type ScoredVec<Move> = Vec<(Move, ValueScore)>;

pub struct MovePicker {
    index: usize,
    moves: ScoredVec<Move>,
}

impl MovePicker {
    pub fn new<F>(moves: &[Move], f: F) -> Self
    where
        F: Fn(Move) -> ValueScore,
    {
        Self { index: 0, moves: Self::decorate_moves_with_score(moves, f) }
    }

    fn decorate_moves_with_score<F>(moves: &[Move], f: F) -> ScoredVec<Move>
    where
        F: Fn(Move) -> ValueScore,
    {
        moves.iter().map(|mov| (*mov, f(*mov))).collect()
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
