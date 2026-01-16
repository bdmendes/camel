use crate::core::position::Position;

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
    use super::*;
    use crate::core::position::fen::START_POSITION;
    use std::str::FromStr;

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
