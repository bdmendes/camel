use arrayvec::ArrayVec;

use crate::core::{
    moves::{Move, see},
    position::{MoveStage, Position, piece::Piece},
};

type ScoredMoveVec = ArrayVec<(Move, i8), 96>;

#[rustfmt::skip]
static QUIET_PSQT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 1,
    1, 2, 4, 4, 4, 4, 2, 1,
    1, 2, 4, 6, 6, 4, 2, 1,
    1, 2, 4, 6, 6, 4, 2, 1,
    1, 2, 4, 4, 4, 4, 2, 1,
    1, 2, 2, 2, 2, 2, 2, 1,
    0, 0, 0, 0, 0, 0, 0, 0,
];

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
            72 + mov.is_capture() as i8
        } else if mov.promotion_piece().is_some() {
            -72
        } else if mov.is_capture() {
            let mvv_lva = self
                .position
                .piece_at(mov.to())
                .unwrap_or(Piece::Pawn)
                .value()
                - self.position.piece_at(mov.from()).unwrap().value();
            if mvv_lva >= 0 {
                36 + mvv_lva
            } else {
                let see = see::see(mov, self.position);
                if see >= 0 { 36 + see } else { -36 + see }
            }
        } else if Some(mov) == self.killer_moves[0] || Some(mov) == self.killer_moves[1] {
            0
        } else if self.position.piece_at(mov.from()).unwrap().value() <= 3 {
            -9 + QUIET_PSQT[mov.to() as usize] - QUIET_PSQT[mov.from() as usize]
        } else {
            -9
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
            let generate = if self.quiesce && !self.position.is_check() {
                MoveStage::CapturesAndPromotions
            } else {
                MoveStage::All
            };
            self.moves = self
                .position
                .moves(generate)
                .iter()
                .filter(|mov| Some(**mov) != self.hash_move)
                .map(|&mov| (mov, self.move_value(mov)))
                .collect();
            if generate == MoveStage::CapturesAndPromotions {
                self.moves.retain(|(_, score)| *score >= 0);
            }
        }

        self.find_next_max_and_swap()
    }
}

#[cfg(test)]
mod tests {
    use super::MovePicker;
    use crate::core::{
        moves::{Move, MoveFlag, see},
        position::{MoveStage, Position, fen::START_POSITION, square::Square},
    };
    use std::{str::FromStr, sync::OnceLock};

    static MOCK_POSITION: OnceLock<Position> = OnceLock::new();

    fn mocks<'a>() -> (&'a Position, MovePicker<'a>) {
        let position = MOCK_POSITION.get_or_init(|| {
            Position::from_str("3rk1nr/1p3pbp/p1npb1pP/4p1q1/P1B1P3/8/1PP2PP1/RNBQNRK1 w k - 2 15")
                .unwrap()
        });
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
    fn no_repeated_hash() {
        let position = Position::from_str(START_POSITION).unwrap();
        let moves = position.moves(MoveStage::All);
        let picker = MovePicker::new(&position, false, Some(moves[0]), [None, None]);
        assert_eq!(picker.collect::<Vec<_>>().len(), moves.len());
    }

    #[test]
    fn quiesce_only_winning_captures() {
        let position = Position::from_str(
            "r1bq1rk1/pp2bppp/2n1p3/2Pp4/2P1n3/P3PN2/1P1NBPPP/R1BQ1RK1 b - - 0 10",
        )
        .unwrap();
        let picker = MovePicker::new(&position, true, None, [None, None]);
        for mov in picker {
            assert!(mov.is_capture());
            assert!(see::see(mov, &position) >= 0);
        }
    }

    #[test]
    fn quiesce_check() {
        let position = Position::from_str("8/6pp/p1n1k3/8/1ppKN3/5P1P/PP4P1/8 w - - 1 34").unwrap();
        let picker = MovePicker::new(&position, true, None, [None, None]);
        let moves = position.moves(MoveStage::All);
        assert_eq!(picker.collect::<Vec<_>>().len(), moves.len());
    }

    #[test]
    fn queen_promotion_first() {
        let position = Position::from_str("8/5P2/8/7p/8/1Kp5/2N3kP/8 w - - 1 51").unwrap();
        let mut picker = MovePicker::new(&position, false, None, [None, None]);
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::F7, Square::F8, MoveFlag::QueenPromotion))
        );
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
        let (_, mut picker) = mocks();
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::C1, Square::G5, MoveFlag::Capture))
        );
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::H6, Square::G7, MoveFlag::Capture))
        );
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::C4, Square::E6, MoveFlag::Capture))
        );
        assert!(!picker.next().unwrap().is_capture());
    }

    #[test]
    fn losing_captures_last() {
        let (position, mut picker) = mocks();
        let number_of_moves = position.moves(MoveStage::All).len();
        for _ in 0..(number_of_moves - 2) {
            picker.next();
        }
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::C4, Square::A6, MoveFlag::Capture))
        );
        assert_eq!(
            picker.next(),
            Some(Move::new(Square::D1, Square::D6, MoveFlag::Capture))
        );
        assert!(picker.next().is_none());
    }

    #[test]
    fn killers_after_winning_captures() {
        let (_, mut picker) = mocks();
        let killers = picker.killer_moves;
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(picker.next().unwrap().is_capture());
        assert!(killers.contains(&picker.next()));
        assert!(killers.contains(&picker.next()));
    }

    #[test]
    fn quiet_center_heuristic() {
        let (_, picker) = mocks();
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
}
