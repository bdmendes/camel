use crate::{
    moves::Move,
    position::{Position, piece::Piece},
    search::see::static_exchange,
};

pub struct MoveEvaluator<'a> {
    position: &'a Position,
    hash_move: Option<Move>,
    killer_moves: [Option<Move>; 2],
}

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

impl<'a> MoveEvaluator<'a> {
    pub fn new(position: &'a Position, hash_move: Option<Move>, killer_moves: [Option<Move>; 2]) -> Self {
        Self {
            position,
            hash_move,
            killer_moves,
        }
    }

    pub fn evaluate(&self, mov: Move) -> i8 {
        if Some(mov) == self.hash_move {
            i8::MAX
        } else if mov.promotion_piece() == Some(Piece::Queen) {
            72 + mov.is_capture() as i8
        } else if mov.promotion_piece().is_some() {
            -72
        } else if mov.is_capture() {
            let see = static_exchange(self.position, mov);
            if see >= 0 {
                let mvv_lva = self.position.piece_at(mov.to()).unwrap_or(Piece::Pawn).value()
                    - self.position.piece_at(mov.from()).unwrap().value();
                32 + see + mvv_lva
            } else {
                -16 + see
            }
        } else if Some(mov) == self.killer_moves[0] || Some(mov) == self.killer_moves[1] {
            0
        } else if self.position.piece_at(mov.from()).unwrap().value() <= 3 {
            -9 + QUIET_PSQT[mov.to() as usize] - QUIET_PSQT[mov.from() as usize]
        } else {
            -9
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{Move, MoveFlag},
        position::{MoveStage, Position, fen::START_POSITION, square::Square},
        search::{move_evaluator::MoveEvaluator, see::static_exchange},
    };
    use std::str::FromStr;

    fn mocks() -> (Position, [Option<Move>; 2]) {
        let position = Position::from_str("3rk1nr/1p3pbp/p1npb1pP/4p1q1/P1B1P3/8/1PP2PP1/RNBQNRK1 w k - 2 15").unwrap();
        let killers = [
            Some(Move::new(Square::E1, Square::F3, MoveFlag::Quiet)),
            Some(Move::new(Square::C1, Square::E3, MoveFlag::Quiet)),
        ];
        (position, killers)
    }

    fn moves(
        position: Position,
        only_captures: bool,
        hash_move: Option<Move>,
        killer_moves: [Option<Move>; 2],
    ) -> Vec<Move> {
        let evaluator = MoveEvaluator::new(&position, hash_move, killer_moves);
        let mut moves = position.moves(if only_captures { MoveStage::CapturesAndPromotions } else { MoveStage::All });
        let mut scored_moves = moves
            .into_iter()
            .map(|mov| (mov, evaluator.evaluate(mov)))
            .collect::<Vec<_>>();
        scored_moves.sort_by(|a, b| b.1.cmp(&a.1));
        scored_moves.into_iter().map(|(mov, _)| mov).collect::<Vec<_>>()
    }

    #[test]
    fn hash_first() {
        let position = Position::from_str(START_POSITION).unwrap();
        let hash_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let vec = moves(position, false, Some(hash_move), [None, None]);
        assert_eq!(vec.len(), 20);
        assert_eq!(vec[0], hash_move);
    }

    #[test]
    fn discard_illegal_hash_move() {
        let position = Position::from_str("r1b2rk1/1p1n1ppp/4p3/p1P1bq2/3P4/1Q2BN2/2N2PPP/R3R1K1 b - -").unwrap();
        let hash_move = Move::new(Square::E5, Square::F6, MoveFlag::EnpassantCapture);
        let vec = moves(position, false, Some(hash_move), [None, None]);
        assert_ne!(vec[0], hash_move);
    }

    #[test]
    fn queen_promotion_first() {
        let position = Position::from_str("8/5P2/8/7p/8/1Kp5/2N3kP/8 w - - 1 51").unwrap();
        let vec = moves(position, false, None, [None, None]);
        assert_eq!(vec[0], Move::new(Square::F7, Square::F8, MoveFlag::QueenPromotion));
        assert!(vec[1].promotion_piece().is_none());
    }

    #[test]
    fn underpromotions_last() {
        let position = Position::from_str("8/5P2/8/7p/8/1Kp5/2N3kP/8 w - - 1 51").unwrap();
        let mut vec = moves(position, false, None, [None, None]);
        let number_of_moves = position.moves(MoveStage::All).len();
        for _ in 0..(number_of_moves - 3) {
            vec.remove(0);
        }
        assert!(vec[0].promotion_piece().is_some());
        assert!(vec[1].promotion_piece().is_some());
        assert!(vec[2].promotion_piece().is_some());
        assert!(vec.len() == 3);
    }

    #[test]
    fn winning_captures_first() {
        let (position, killers) = mocks();
        let vec = moves(position, false, None, killers);
        assert_eq!(vec[0], Move::new(Square::C1, Square::G5, MoveFlag::Capture));
        assert_eq!(vec[1], Move::new(Square::H6, Square::G7, MoveFlag::Capture));
        assert_eq!(vec[2], Move::new(Square::C4, Square::E6, MoveFlag::Capture));
    }

    #[test]
    fn good_captures_then_killers_then_bad_captures() {
        let (position, killers) = mocks();
        let vec = moves(position, false, None, killers);

        let first_killer_idx = vec.iter().position(|mov| killers.contains(&Some(*mov))).unwrap();
        let last_good_capture_idx = vec
            .iter()
            .rposition(|mov| mov.is_capture() && static_exchange(&position, *mov) >= 0)
            .unwrap();
        assert!(last_good_capture_idx < first_killer_idx);

        let first_bad_capture_idx = vec
            .iter()
            .position(|mov| mov.is_capture() && static_exchange(&position, *mov) < 0)
            .unwrap();
        assert!(first_killer_idx < first_bad_capture_idx);
    }

    #[test]
    fn quiet_center_heuristic() {
        let (position, killers) = mocks();
        let vec = moves(position, false, None, killers);

        let knight_to_corner_idx = vec
            .iter()
            .position(|mov| mov.from() == Square::B1 && mov.to() == Square::A3)
            .unwrap();
        let knight_to_center_idx = vec
            .iter()
            .position(|mov| mov.from() == Square::B1 && mov.to() == Square::C3)
            .unwrap();
        assert!(knight_to_center_idx < knight_to_corner_idx);

        let bishop_retreat_idx = vec
            .iter()
            .position(|mov| mov.from() == Square::C4 && mov.to() == Square::E2)
            .unwrap();
        let bishop_to_center_idx = vec
            .iter()
            .position(|mov| mov.from() == Square::C4 && mov.to() == Square::D5)
            .unwrap();
        assert!(bishop_to_center_idx < bishop_retreat_idx);
    }
}
