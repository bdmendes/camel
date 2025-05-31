use super::{make::make_move, Move, MoveFlag};
use crate::core::{
    bitboard::Bitboard, color::Color, piece::Piece, square::Square, MoveStage, Position,
};
use arrayvec::ArrayVec;
use castle::castle_moves;
use leapers::{king_attackers, king_regular_moves, knight_attackers, knight_moves};
use magics::queen_attacks;
use pawns::{pawn_attackers, pawn_moves};
use sliders::{bishop_moves, diagonal_attackers, file_attackers, queen_moves, rook_moves};

pub mod castle;
pub mod leapers;
pub mod magics;
pub mod pawns;
pub mod sliders;

pub type MoveVec = ArrayVec<Move, 96>;

pub fn generate_moves(position: &Position, stage: MoveStage) -> MoveVec {
    let mut moves = MoveVec::new();

    let our_king = position.pieces_color_bb(Piece::King, position.side_to_move).lsb().unwrap();
    let king_attackers = square_attackers(position, our_king, position.side_to_move.flipped());
    let king_ray = queen_attacks(position, our_king);
    let between_attacker = Bitboard::between(our_king, king_attackers.msb().unwrap_or(our_king));

    king_regular_moves(position, stage, &mut moves);

    if king_attackers.count_ones() <= 1 {
        pawn_moves(position, stage, &mut moves);
        knight_moves(position, stage, &mut moves);
        bishop_moves(position, stage, &mut moves);
        rook_moves(position, stage, &mut moves);
        queen_moves(position, stage, &mut moves);
        if king_attackers.is_empty() {
            castle_moves(position, stage, &mut moves);
        }
    }

    moves.retain(|mov| {
        match mov.flag() {
            MoveFlag::EnpassantCapture | MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => {}
            _ if mov.from() != our_king => {
                // If not capturing the checker or attempting to block, this is not legal.
                if !king_attackers.is_empty()
                    && if mov.is_capture() {
                        !king_attackers.is_set(mov.to())
                    } else {
                        !between_attacker.is_set(mov.to())
                    }
                {
                    return false;
                }

                // If not in check and not removing from king rays, we're sure this is legal.
                if king_attackers.is_empty() && !king_ray.is_set(mov.from()) {
                    return true;
                }
            }
            _ => {}
        };

        let new_position = make_move::<false>(position, *mov);
        !new_position.is_check()
    });

    moves
}

pub fn square_attackers(position: &Position, square: Square, color: Color) -> Bitboard {
    pawn_attackers(position, color, square)
        | knight_attackers(position, color, square)
        | king_attackers(position, color, square)
        | file_attackers(position, color, square)
        | diagonal_attackers(position, color, square)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{core::moves::Move, core::Position};

    use super::{MoveStage, MoveVec};

    fn assert_eq_vec_move(moves: &[Move], expected: &[&str]) {
        assert_eq!(moves.len(), expected.len());
        let mov_strs = moves.iter().map(|m| m.to_string()).collect::<Vec<String>>();
        moves.iter().map(|m| m.to_string()).for_each(|m| {
            assert!(expected.contains(&m.as_str()), "got: {:?}, expected: {:?}", mov_strs, expected)
        });
    }

    pub fn assert_staged_moves(
        position: &str,
        function: fn(&Position, MoveStage, &mut MoveVec),
        expected: [Vec<&str>; 3],
    ) {
        let position = Position::from_str(position).unwrap();

        let mut moves1 = MoveVec::new();
        function(&position, MoveStage::All, &mut moves1);
        assert_eq_vec_move(&moves1, &expected[0]);

        let mut moves2 = MoveVec::new();
        function(&position, MoveStage::CapturesAndPromotions, &mut moves2);
        assert_eq_vec_move(&moves2, &expected[1]);

        let mut moves3 = MoveVec::new();
        function(&position, MoveStage::Quiet, &mut moves3);
        assert_eq_vec_move(&moves3, &expected[2]);
    }
}
