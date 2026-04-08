use crate::position::{Position, color::Color, hash::ZobristHash};

#[derive(Debug, Clone)]
pub struct GameHistory {
    hashes: [Vec<ZobristHash>; 2],
    barriers: [Vec<usize>; 2],
}

impl GameHistory {
    pub fn new(position: &Position) -> Self {
        let mut history = Self {
            hashes: [Vec::with_capacity(32), Vec::with_capacity(32)],
            barriers: [Vec::with_capacity(16), Vec::with_capacity(16)],
        };
        history.push(position, false);
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
        let handle = position.side_to_move() as usize;
        let (hashes, barriers) = (&self.hashes[handle], &self.barriers[handle]);
        let first_idx = *barriers.last().unwrap_or(&0);
        let hash = position.hash();
        hashes[first_idx..].iter().filter(|&&h| h == hash).count()
    }

    pub fn push(&mut self, position: &Position, reversible: bool) {
        let handle = position.side_to_move() as usize;
        let (hashes, barriers) = (&mut self.hashes[handle], &mut self.barriers[handle]);
        if !reversible || hashes.is_empty() {
            barriers.push(hashes.len());
        }
        hashes.push(position.hash());
    }

    pub fn pop(&mut self, color: Color) {
        let handle = color as usize;
        let (hashes, barriers) = (&mut self.hashes[handle], &mut self.barriers[handle]);
        if barriers.last().unwrap() + 1 == hashes.len() {
            barriers.pop();
        }
        hashes.pop();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::str::FromStr;

    use crate::{
        position::{
            Position,
            color::Color,
            fen::{Fen, START_POSITION},
        },
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

    #[test]
    fn push_pop_pair() {
        let position = Position::from_str(START_POSITION).unwrap();
        let position2 = position.make_move_str("g1f3").unwrap();
        let position3 = position2.make_move_str("g8f6").unwrap();

        let mut history = GameHistory::new(&position);
        history.push(&position2, true);
        history.push(&position3, true);
        history.pop(position3.side_to_move());
        history.pop(position2.side_to_move());
    }

    #[test]
    fn push_pop_barrier() {
        let position = Position::from_str(START_POSITION).unwrap();
        let mut history = GameHistory::new(&position);

        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position.make_move_str("e2e4").unwrap()), 0);

        history.push(&position, true);
        assert_eq!(history.seen(&position), 2);
        history.push(&position, true);
        assert_eq!(history.seen(&position), 3);

        history.push(&position, false);
        assert_eq!(history.seen(&position), 1);

        history.pop(Color::White);
        assert_eq!(history.seen(&position), 3);
        history.pop(Color::White);
        assert_eq!(history.seen(&position), 2);
        history.pop(Color::White);
        assert_eq!(history.seen(&position), 1);
        history.pop(Color::White);
        assert_eq!(history.seen(&position), 0);
    }

    #[test]
    fn mixed_positions() {
        let position = Position::from_str(START_POSITION).unwrap();
        let position2 = position.make_move_str("d2d4").unwrap();
        let position3 = position2.make_move_str("g8f6").unwrap();
        let position4 = position3.make_move_str("c2c4").unwrap();
        let position5 = position4.make_move_str("g7g6").unwrap();
        let mut history = GameHistory::new(&position);

        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 0);

        history.push(&position2, false);
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 1);

        history.push(&position3, true);
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 1);
        assert_eq!(history.seen(&position3), 1);

        history.push(&position4, false);
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 0);
        assert_eq!(history.seen(&position3), 1);
        assert_eq!(history.seen(&position4), 1);

        history.push(&position5, false);
        assert_eq!(history.seen(&position), 0);
        assert_eq!(history.seen(&position2), 0);
        assert_eq!(history.seen(&position3), 0);
        assert_eq!(history.seen(&position4), 1);
        assert_eq!(history.seen(&position5), 1);

        history.pop(position5.side_to_move());
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 0);
        assert_eq!(history.seen(&position3), 1);
        assert_eq!(history.seen(&position4), 1);
        assert_eq!(history.seen(&position5), 0);

        history.pop(position4.side_to_move());
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 1);
        assert_eq!(history.seen(&position3), 1);
        assert_eq!(history.seen(&position4), 0);
        assert_eq!(history.seen(&position5), 0);
    }
}
