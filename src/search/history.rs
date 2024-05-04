use crate::position::{board::ZobristHash, Position};

#[derive(Debug, Copy, Clone)]
pub struct HistoryEntry {
    pub board_hash: ZobristHash,
    pub reversible: bool,
}

pub struct BranchHistory(pub Vec<HistoryEntry>);

impl BranchHistory {
    pub fn visit_position(&mut self, position: &Position, reversible: bool) {
        self.0.push(HistoryEntry { board_hash: position.board.zobrist_hash(), reversible });
    }

    pub fn leave_position(&mut self) {
        self.0.pop();
    }

    pub fn repeated(&self, position: &Position) -> u8 {
        let board_hash = position.board.zobrist_hash();
        self.0
            .iter()
            .rev()
            .skip_while(|entry| entry.board_hash != board_hash) // Skip until we find the current position.
            .step_by(2) // We can only repeat when it is our turn to move.
            .take_while(|entry| entry.reversible || entry.board_hash == board_hash)
            .filter(|entry| entry.board_hash == board_hash)
            .count() as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        position::{
            fen::{FromFen, START_FEN},
            Position,
        },
        search::history::BranchHistory,
    };

    #[test]
    fn repeated_times() {
        let mut history = BranchHistory(Vec::new());

        let mut position = Position::from_fen(START_FEN).unwrap();
        history.visit_position(&position, true);

        position = position.make_move_str("e2e4").unwrap();
        history.visit_position(&position, false);

        position = position.make_move_str("e7e5").unwrap();
        history.visit_position(&position, false);

        assert_eq!(history.repeated(&position), 1);

        position = position.make_move_str("g1f3").unwrap();
        history.visit_position(&position, true);

        position = position.make_move_str("b8c6").unwrap();
        history.visit_position(&position, true);

        position = position.make_move_str("f3g1").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 1);

        position = position.make_move_str("c6b8").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 2);

        position = position.make_move_str("g1f3").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 2);

        position = position.make_move_str("b8c6").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 2);

        position = position.make_move_str("f3g1").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 2);

        position = position.make_move_str("c6b8").unwrap();
        history.visit_position(&position, true);

        assert_eq!(history.repeated(&position), 3);
    }
}
