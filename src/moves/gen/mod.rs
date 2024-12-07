use super::Move;
use crate::position::{bitboard::Bitboard, color::Color, square::Square, Position};
use leapers::{king_regular_moves, knight_moves};
use pawns::{pawn_attackers, pawn_moves};

mod leapers;
mod pawns;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MoveStage {
    All,
    CapturesAndPromotions,
    Quiet,
}

pub fn generate_moves(position: &Position, stage: MoveStage) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);

    pawn_moves(position, stage, &mut moves);
    knight_moves(position, stage, &mut moves);
    king_regular_moves(position, stage, &mut moves);

    moves
}

pub fn square_attackers(position: &Position, square: Square, color: Color) -> Bitboard {
    pawn_attackers(position, color, square)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{moves::Move, position::Position};

    use super::MoveStage;

    fn assert_eq_vec_move(moves: &[Move], expected: &[&str]) {
        assert_eq!(moves.len(), expected.len());
        let mov_strs = moves.iter().map(|m| m.to_string()).collect::<Vec<String>>();
        moves.iter().map(|m| m.to_string()).for_each(|m| {
            assert!(
                expected.contains(&m.as_str()),
                "got: {:?}, expected: {:?}",
                mov_strs,
                expected
            )
        });
    }

    pub fn assert_staged_moves(
        position: &str,
        function: fn(&Position, MoveStage, &mut Vec<Move>),
        expected: [Vec<&str>; 3],
    ) {
        let position = Position::from_str(position).unwrap();

        let mut moves1 = Vec::new();
        function(&position, MoveStage::All, &mut moves1);
        assert_eq_vec_move(&moves1, &expected[0]);

        let mut moves2 = Vec::new();
        function(&position, MoveStage::CapturesAndPromotions, &mut moves2);
        assert_eq_vec_move(&moves2, &expected[1]);

        let mut moves3 = Vec::new();
        function(&position, MoveStage::Quiet, &mut moves3);
        assert_eq_vec_move(&moves3, &expected[2]);
    }
}
