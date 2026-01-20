use crate::position::Position;

pub struct GameHistory {
    data: Vec<Entry>,
}

struct Entry {
    hash_ms16: u16,
    is_reversible: bool,
}

impl Default for GameHistory {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(64),
        }
    }
}

impl GameHistory {
    pub fn single(position: &Position) -> GameHistory {
        let mut history = GameHistory::default();
        history.push(position, false);
        history
    }

    pub fn from_moves(position: &Position, moves: &[&str]) -> Option<(GameHistory, Position)> {
        let mut history = GameHistory::single(position);
        let mut position = *position;
        for mov in moves {
            if let Some(m) = position.get_move_str(mov) {
                let reversible = m.is_reversible(&position);
                position = position.make_move(m);
                history.push(&position, reversible);
            } else {
                return None;
            }
        }
        Some((history, position))
    }

    pub fn push(&mut self, position: &Position, is_reversible: bool) {
        self.data.push(Entry {
            hash_ms16: position.hash().ms16(),
            is_reversible,
        })
    }

    pub fn pop(&mut self) {
        self.data.pop();
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn seen(&self, position: &Position) -> u8 {
        let mut count = 0;
        for entry in self.data.iter().rev() {
            if entry.hash_ms16 == position.hash().ms16() {
                count += 1;
            }
            if !entry.is_reversible {
                break;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::position::fen::{Fen, START_POSITION};
    use std::str::FromStr;

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
        let moves = moves.split(" ").collect::<Vec<_>>();
        let position = Position::try_from(fen).unwrap();
        let history = GameHistory::from_moves(&position, &moves);
        assert_eq!(history.is_some(), valid);
        if let Some((history, _)) = history {
            assert_eq!(history.data.len(), moves.len() + 1);
        }
    }

    #[test]
    fn push_pop() {
        let mut history = GameHistory::default();
        assert_eq!(history.data.len(), 0);

        let position = Position::from_str(START_POSITION).unwrap();
        history.push(&position, false);
        assert_eq!(history.data.len(), 1);
        history.push(&position.make_move_str("e2e4").unwrap(), false);
        assert_eq!(history.data.len(), 2);

        history.pop();
        assert_eq!(history.data.len(), 1);
        history.pop();
        assert_eq!(history.data.len(), 0);
        history.pop();
        assert_eq!(history.data.len(), 0);
    }

    #[test]
    fn seen() {
        let mut history = GameHistory::default();
        let position = Position::from_str(START_POSITION).unwrap();

        history.push(&position, false);
        assert_eq!(history.seen(&position), 1);

        let position2 = position.make_move_str("e2e4").unwrap();
        history.push(&position2, false);
        assert_eq!(history.seen(&position2), 1);

        history.pop();
        assert_eq!(history.seen(&position), 1);
        assert_eq!(history.seen(&position2), 0);
    }

    #[test]
    fn short_circuits() {
        let mut history = GameHistory::default();
        let position = Position::from_str(START_POSITION).unwrap();

        history.push(&position, true);
        assert_eq!(history.seen(&position), 1);

        history.push(&position, true);
        assert_eq!(history.seen(&position), 2);

        history.push(&position, false);
        assert_eq!(history.seen(&position), 1);
    }

    #[test]
    fn clear() {
        let mut history = GameHistory::default();
        let position = Position::from_str(START_POSITION).unwrap();

        history.push(&position, true);
        history.push(&position, true);
        assert_eq!(history.data.len(), 2);

        history.clear();
        assert_eq!(history.data.len(), 0);
        assert_eq!(history.seen(&position), 0);
    }
}
