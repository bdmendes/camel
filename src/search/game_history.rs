use crate::position::{Position, color::Color, hash::ZobristHash};

pub struct GameHistory {
    hashes: [Vec<ZobristHash>; 2],
    barriers: [Vec<usize>; 2],
    side_to_move: Color,
}

impl GameHistory {
    pub fn new(position: &Position) -> Self {
        let sign = position.side_to_move().sign() as usize;
        let mut history = Self {
            hashes: [Vec::with_capacity(32), Vec::with_capacity(32)],
            barriers: [Vec::with_capacity(16), Vec::with_capacity(16)],
            side_to_move: position.side_to_move(),
        };
        history.hashes[sign].push(position.hash());
        history.barriers[sign].push(0);
        history
    }

    pub fn from_moves(position: &Position, moves: &[&str]) -> Option<(Self, Position)> {
        let mut history = Self::new(position);
        let mut position = *position;
        for mov in moves {
            if let Some(m) = position.get_move_str(mov) {
                let next = position.make_move(m);
                history.push(&next, m.is_reversible(&position));
                position = next;
            } else {
                return None;
            }
        }
        Some((history, position))
    }

    pub fn seen(&self, position: &Position) -> usize {
        let sign = position.side_to_move().sign() as usize;
        let (hashes, barriers) = (&self.hashes[sign], &self.barriers[sign]);
        let first_idx = *barriers.last().unwrap_or(&0);
        let hash = position.hash();
        hashes[first_idx..].iter().filter(|&&h| h == hash).count()
    }

    pub fn push(&mut self, position: &Position, reversible: bool) {
        let sign = position.side_to_move().sign() as usize;
        let (hashes, barriers) = (&mut self.hashes[sign], &mut self.barriers[sign]);
        if !reversible || hashes.is_empty() {
            barriers.push(hashes.len());
        }
        hashes.push(position.hash());
        self.side_to_move = position.side_to_move();
    }

    pub fn pop(&mut self) {
        let sign = self.side_to_move.sign() as usize;
        let (hashes, barriers) = (&mut self.hashes[sign], &mut self.barriers[sign]);
        if barriers.last().unwrap() + 1 == hashes.len() {
            barriers.pop();
        }
        hashes.pop();
    }
}
