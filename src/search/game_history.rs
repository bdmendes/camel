use crate::position::{Position, color::Color, hash::ZobristHash};

pub struct GameHistory {
    hashes: [Vec<ZobristHash>; 2],
    barriers: [Vec<usize>; 2],
    side_to_move: Color,
}

impl GameHistory {
    pub fn new(position: &Position) -> Self {
        let sign = position.side_to_move() as usize;
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
        let sign = position.side_to_move() as usize;
        let (hashes, barriers) = (&self.hashes[sign], &self.barriers[sign]);
        let first_idx = *barriers.last().unwrap_or(&0);
        let hash = position.hash();
        hashes[first_idx..].iter().filter(|&&h| h == hash).count()
    }

    pub fn push(&mut self, position: &Position, reversible: bool) {
        let sign = position.side_to_move() as usize;
        let (hashes, barriers) = (&mut self.hashes[sign], &mut self.barriers[sign]);
        if !reversible || hashes.is_empty() {
            barriers.push(hashes.len());
        }
        hashes.push(position.hash());
        self.side_to_move = position.side_to_move();
    }

    pub fn pop(&mut self) {
        let sign = self.side_to_move as usize;
        let (hashes, barriers) = (&mut self.hashes[sign], &mut self.barriers[sign]);
        if barriers.last().unwrap() + 1 == hashes.len() {
            barriers.pop();
        }
        hashes.pop();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        position::{Position, fen::Fen},
        search::game_history::GameHistory,
    };

    #[rstest]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e2e4", true)]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e2e4 d7d5", true)]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e2e4 d7d5 g1f3", true)]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e2e5 d7d5 g1f3", false)]
    #[case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "e2e4 d7d5 g1f4", false)]
    #[case("rn1q1rk1/pp1bbppp/2p1pn2/8/2QP4/2N2NP1/PP2PPBP/R1B2RK1 b - -", "d8b6 e2e4", true)]
    #[case("rn1q1rk1/pp1bbppp/2p1pn2/8/2QP4/2N2NP1/PP2PPBP/R1B2RK1 b - -", "d8b6 e2e5", false)]
    #[case("rn1q1rk1/pp1bbppp/2p1pn2/8/2QP4/2N2NP1/PP2PPBP/R1B2RK1 b - -", "d8b6 e2e5 f8e8", false)]
    fn from_moves(#[case] fen: Fen, #[case] moves: &str, #[case] valid: bool) {
        let moves = moves.split_whitespace().collect::<Vec<_>>();
        let position = Position::try_from(fen).unwrap();
        let history = GameHistory::from_moves(&position, &moves);
        assert_eq!(history.is_some(), valid);
        if let Some((history, _)) = history {
            assert_eq!(history.hashes[0].len() + history.hashes[1].len(), moves.len() + 1);
        }
    }
}
