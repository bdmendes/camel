use smallvec::SmallVec;

use crate::{
    moves::Move,
    position::{MoveStage, Position, piece::Piece},
};

type ScoredMoveVec = SmallVec<[(Move, i8); 64]>;

#[rustfmt::skip]
const QUIET_PSQT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 1,
    1, 2, 4, 4, 4, 4, 2, 1,
    1, 2, 4, 6, 6, 4, 2, 1,
    1, 2, 4, 6, 6, 4, 2, 1,
    1, 2, 4, 4, 4, 4, 2, 1,
    1, 2, 2, 2, 2, 2, 2, 1,
    0, 0, 0, 0, 0, 0, 0, 0,
];

pub struct MovePicker {
    moves: ScoredMoveVec,
    current: usize,
}

impl MovePicker {
    pub fn new(
        position: &Position,
        captures_only: bool,
        hash_move: Option<Move>,
        killer_moves: [Option<Move>; 2],
    ) -> Self {
        let move_value = |mov: Move| -> i8 {
            if Some(mov) == hash_move {
                i8::MAX
            } else if mov.promotion_piece() == Some(Piece::Queen) {
                72 + mov.is_capture() as i8
            } else if mov.promotion_piece().is_some() {
                -72
            } else if mov.is_capture() {
                48 + position.piece_at(mov.to()).unwrap_or(Piece::Pawn).value()
                    - position.piece_at(mov.from()).unwrap().value()
            } else if Some(mov) == killer_moves[0] || Some(mov) == killer_moves[1] {
                0
            } else if position.piece_at(mov.from()).unwrap().value() <= 3 {
                -9 + QUIET_PSQT[mov.to() as usize] - QUIET_PSQT[mov.from() as usize]
            } else {
                -9
            }
        };

        let generate = if captures_only { MoveStage::CapturesAndPromotions } else { MoveStage::All };
        let moves = position
            .moves(generate)
            .iter()
            .map(|&mov| (mov, move_value(mov)))
            .collect::<ScoredMoveVec>();

        Self { moves, current: 0 }
    }

    pub fn len(&self) -> usize {
        self.moves.len().saturating_sub(self.current)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Iterator for MovePicker {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.moves.len() {
            return None;
        }

        let mut best_score = self.moves[self.current].1;

        for i in (self.current + 1)..self.moves.len() {
            if self.moves[i].1 > best_score {
                best_score = self.moves[i].1;
                self.moves.swap(i, self.current);
            }
        }

        self.current += 1;
        Some(self.moves[self.current - 1].0)
    }
}

#[cfg(test)]
mod tests {
    use super::MovePicker;
    use crate::{
        moves::{Move, MoveFlag},
        position::{MoveStage, Position, fen::START_POSITION, square::Square},
    };
    use std::str::FromStr;

    fn mocks() -> (Position, MovePicker, [Option<Move>; 2]) {
        let position = Position::from_str("3rk1nr/1p3pbp/p1npb1pP/4p1q1/P1B1P3/8/1PP2PP1/RNBQNRK1 w k - 2 15").unwrap();
        let killers = [
            Some(Move::new(Square::E1, Square::F3, MoveFlag::Quiet)),
            Some(Move::new(Square::C1, Square::E3, MoveFlag::Quiet)),
        ];
        let picker = MovePicker::new(&position, false, None, killers);
        (position, picker, killers)
    }

    #[test]
    fn no_moves() {
        let position = Position::from_str("8/k5K1/8/8/8/8/1Q6/Q7 b - - 16 65").unwrap();

        let picker = MovePicker::new(&position, false, None, [None, None]);
        assert_eq!(picker.len(), 0);
        assert!(picker.is_empty());

        let mut picker = picker.peekable();
        assert!(picker.peek().is_none());
        assert!(picker.next().is_none());
    }

    #[test]
    fn hash_first() {
        let position = Position::from_str(START_POSITION).unwrap();
        let hash_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mut picker = MovePicker::new(&position, false, Some(hash_move), [None, None]);
        assert_eq!(picker.len(), 20);
        assert_eq!(picker.next(), Some(hash_move));
        assert_eq!(picker.len(), 19);
    }

    #[test]
    fn discard_illegal_hash_move() {
        let position = Position::from_str("r1b2rk1/1p1n1ppp/4p3/p1P1bq2/3P4/1Q2BN2/2N2PPP/R3R1K1 b - -").unwrap();
        let hash_move = Move::new(Square::E5, Square::F6, MoveFlag::EnpassantCapture);
        let mut picker = MovePicker::new(&position, false, Some(hash_move), [None, None]);
        assert_ne!(picker.next(), Some(hash_move));
    }

    #[test]
    fn no_repeated_hash() {
        let position = Position::from_str(START_POSITION).unwrap();
        let moves = position.moves(MoveStage::All);
        let picker = MovePicker::new(&position, false, Some(moves[0]), [None, None]);
        assert_eq!(picker.collect::<Vec<_>>().len(), moves.len());
    }

    #[test]
    fn queen_promotion_first() {
        let position = Position::from_str("8/5P2/8/7p/8/1Kp5/2N3kP/8 w - - 1 51").unwrap();
        let mut picker = MovePicker::new(&position, false, None, [None, None]);
        assert_eq!(picker.next(), Some(Move::new(Square::F7, Square::F8, MoveFlag::QueenPromotion)));
        assert!(picker.next().unwrap().promotion_piece().is_none());
    }

    #[test]
    fn underpromotions_last() {
        let position = Position::from_str("8/5P2/8/7p/8/1Kp5/2N3kP/8 w - - 1 51").unwrap();
        let mut picker = MovePicker::new(&position, false, None, [None, None]);
        let number_of_moves = position.moves(MoveStage::All).len();
        for _ in 0..(number_of_moves - 3) {
            picker.next();
        }
        assert!(picker.next().unwrap().promotion_piece().is_some());
        assert!(picker.next().unwrap().promotion_piece().is_some());
        assert!(picker.next().unwrap().promotion_piece().is_some());
        assert!(picker.next().is_none());
    }

    #[test]
    fn winning_captures_first() {
        let (_, mut picker, _) = mocks();
        assert_eq!(picker.next(), Some(Move::new(Square::C1, Square::G5, MoveFlag::Capture)));
        assert_eq!(picker.next(), Some(Move::new(Square::H6, Square::G7, MoveFlag::Capture)));
        assert_eq!(picker.next(), Some(Move::new(Square::C4, Square::E6, MoveFlag::Capture)));
    }

    #[test]
    fn killers_after_captures() {
        let (_, mut picker, killers) = mocks();
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(killers.contains(&picker.next()));
        assert!(killers.contains(&picker.next()));
        assert!(!picker.next().unwrap().is_capture());
    }

    #[test]
    fn quiet_center_heuristic() {
        let (_, picker, _) = mocks();
        let moves = picker.collect::<Vec<_>>();

        let knight_to_corner_idx = moves
            .iter()
            .position(|mov| mov.from() == Square::B1 && mov.to() == Square::A3)
            .unwrap();
        let knight_to_center_idx = moves
            .iter()
            .position(|mov| mov.from() == Square::B1 && mov.to() == Square::C3)
            .unwrap();
        assert!(knight_to_center_idx < knight_to_corner_idx);

        let bishop_retreat_idx = moves
            .iter()
            .position(|mov| mov.from() == Square::C4 && mov.to() == Square::E2)
            .unwrap();
        let bishop_to_center_idx = moves
            .iter()
            .position(|mov| mov.from() == Square::C4 && mov.to() == Square::D5)
            .unwrap();
        assert!(bishop_to_center_idx < bishop_retreat_idx);
    }

    #[test]
    fn only_captures() {
        let position = Position::from_str("3rrkR1/1p6/p5p1/2p5/1qb1B3/2N3P1/PP2PP2/2KR4 b - - 1 26").unwrap();
        let mut picker = MovePicker::new(&position, true, None, [None, None]);
        assert_eq!(picker.next(), Some(Move::new(Square::C4, Square::G8, MoveFlag::Capture)));
        assert_eq!(picker.next(), Some(Move::new(Square::F8, Square::G8, MoveFlag::Capture)));
        assert_eq!(picker.next(), None);
    }
}
