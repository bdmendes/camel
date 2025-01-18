use arrayvec::ArrayVec;

use crate::core::{
    moves::{see, Move},
    piece::Piece,
    MoveStage, Position,
};

type ScoredMoveVec = ArrayVec<(Move, i8), 96>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerStage {
    HashMove,
    OtherMoves,
}

pub struct MovePicker<'a> {
    position: &'a Position,
    quiesce: bool,
    hash_move: Option<Move>,
    killer_moves: [Option<Move>; 2],
    stage: PickerStage,
    moves: ScoredMoveVec,
    current: usize,
}

impl<'a> MovePicker<'a> {
    pub fn new(
        position: &'a Position,
        quiesce: bool,
        hash_move: Option<Move>,
        killer_moves: [Option<Move>; 2],
    ) -> Self {
        Self {
            position,
            quiesce,
            hash_move,
            killer_moves,
            stage: PickerStage::HashMove,
            moves: ScoredMoveVec::new(),
            current: 0,
        }
    }

    fn move_value(&self, mov: Move) -> i8 {
        if mov.promotion_piece() == Some(Piece::Queen) {
            18 + mov.is_capture() as i8
        } else if mov.is_capture() {
            let see = see::see(mov, self.position);
            if see >= 0 {
                9 + see
            } else {
                -9 + see
            }
        } else if Some(mov) == self.killer_moves[0] || Some(mov) == self.killer_moves[1] {
            0
        } else {
            -1
        }
    }

    fn find_next_max_and_swap(&mut self) -> Option<Move> {
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

impl Iterator for MovePicker<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stage == PickerStage::HashMove {
            self.stage = PickerStage::OtherMoves;
            return self.hash_move.or_else(|| self.next());
        }

        if self.moves.is_empty() {
            self.moves = self
                .position
                .moves(if self.quiesce { MoveStage::CapturesAndPromotions } else { MoveStage::All })
                .iter()
                .filter(|mov| Some(**mov) != self.hash_move)
                .map(|&mov| (mov, self.move_value(mov)))
                .collect();
        }

        self.find_next_max_and_swap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::core::{
        fen::START_POSITION,
        moves::{Move, MoveFlag},
        square::Square,
        MoveStage, Position,
    };

    use super::MovePicker;

    fn mocks<'a>() -> (&'a Position, MovePicker<'a>) {
        let position = Box::leak(Box::new(
            Position::from_str("r3kbnr/1p3ppp/p1npb3/4p1q1/2B1P3/8/PPP2PPP/RNBQNRK1 w kq - 6 9")
                .unwrap(),
        ));
        let killers = [
            Some(Move::new(Square::E1, Square::F3, MoveFlag::Quiet)),
            Some(Move::new(Square::C1, Square::E3, MoveFlag::Quiet)),
        ];
        let picker = MovePicker::new(position, false, None, killers);
        (position, picker)
    }

    #[test]
    fn no_moves() {
        let position = Position::from_str("8/k5K1/8/8/8/8/1Q6/Q7 b - - 16 65").unwrap();
        let mut picker = MovePicker::new(&position, false, None, [None, None]).peekable();
        assert!(picker.peek().is_none());
        assert!(picker.next().is_none());
    }

    #[test]
    fn hash_first() {
        let position = Position::from_str(START_POSITION).unwrap();
        let hash_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mut picker = MovePicker::new(&position, false, Some(hash_move), [None, None]);
        assert_eq!(picker.next(), Some(hash_move));
        assert!(picker.moves.is_empty());
    }

    #[test]
    fn quiesce_only_captures() {
        let position = Position::from_str(
            "r1bq1rk1/pp2bppp/2n1p3/2Pp4/2P1n3/P3PN2/1P1NBPPP/R1BQ1RK1 b - - 0 10",
        )
        .unwrap();
        let picker = MovePicker::new(&position, true, None, [None, None]);
        for mov in picker {
            assert!(mov.is_capture());
        }
    }

    #[test]
    fn winning_captures_first() {
        let (_, mut picker) = mocks();
        assert_eq!(picker.next(), Some(Move::new(Square::C1, Square::G5, MoveFlag::Capture)));
        assert_eq!(picker.next(), Some(Move::new(Square::C4, Square::E6, MoveFlag::Capture)));
        assert!(!picker.next().unwrap().is_capture());
    }

    #[test]
    fn losing_captures_last() {
        let (position, mut picker) = mocks();
        let number_of_moves = position.moves(MoveStage::All).len();
        for _ in 0..(number_of_moves - 2) {
            picker.next();
        }
        assert_eq!(picker.next(), Some(Move::new(Square::C4, Square::A6, MoveFlag::Capture)));
        assert_eq!(picker.next(), Some(Move::new(Square::D1, Square::D6, MoveFlag::Capture)));
        assert!(picker.next().is_none());
    }

    #[test]
    fn killers_after_winning_captures() {
        let (_, mut picker) = mocks();
        let killers = picker.killer_moves;
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(killers.contains(&picker.next()));
        assert!(killers.contains(&picker.next()));
    }
}
