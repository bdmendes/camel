use ctor::ctor;

use crate::{
    moves::{Move, MoveFlag},
    position::{bitboard::Bitboard, color::Color, piece::Piece, square::Square, Position},
};

use super::MoveStage;

#[ctor]
static LAST_RANKS: Bitboard = Bitboard::rank_mask(0) | Bitboard::rank_mask(7);

fn pawn_moves_front(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    let our_pawns =
        position.occupancy_bb(position.side_to_move()) & position.pieces_bb(Piece::Pawn);

    let walk = our_pawns.shift(match position.side_to_move() {
        Color::White => Square::NORTH,
        Color::Black => Square::SOUTH,
    }) & match stage {
        MoveStage::All => !position.occupancy_bb_all(),
        MoveStage::CapturesAndPromotions => !position.occupancy_bb_all() & *LAST_RANKS,
        MoveStage::Quiet => !position.occupancy_bb_all() & !(*LAST_RANKS),
    };

    let origin = match position.side_to_move() {
        Color::White => |sq: Square| sq.shift(Square::SOUTH),
        Color::Black => |sq: Square| sq.shift(Square::NORTH),
    };

    for sq in walk {
        let from = origin(sq);
        if sq.rank() == 0 || sq.rank() == 7 {
            moves.push(Move::new(from, sq, MoveFlag::KnightPromotion));
            moves.push(Move::new(from, sq, MoveFlag::BishopPromotion));
            moves.push(Move::new(from, sq, MoveFlag::RookPromotion));
            moves.push(Move::new(from, sq, MoveFlag::QueenPromotion));
        } else {
            moves.push(Move::new(from, sq, MoveFlag::Quiet));
        }
    }
}

pub fn pawn_moves(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    pawn_moves_front(position, stage, moves);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        moves::gen::{tests::assert_eq_vec_move, MoveStage},
        position::Position,
    };

    use super::pawn_moves_front;

    #[test]
    fn front_1() {
        let position =
            Position::from_str("r1bqkbnr/ppp2ppp/2np4/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4")
                .unwrap();

        let mut moves1 = Vec::new();
        pawn_moves_front(&position, MoveStage::All, &mut moves1);
        assert_eq_vec_move(&moves1, &["a2a3", "b2b3", "c2c3", "d4d5", "g2g3", "h2h3"]);

        let mut moves2 = Vec::new();
        pawn_moves_front(&position, MoveStage::CapturesAndPromotions, &mut moves2);
        assert_eq_vec_move(&moves2, &[]);

        let mut moves3 = Vec::new();
        pawn_moves_front(&position, MoveStage::Quiet, &mut moves3);
        assert_eq_vec_move(&moves3, &["a2a3", "b2b3", "c2c3", "d4d5", "g2g3", "h2h3"]);
    }

    #[test]
    fn front_2() {
        let position = Position::from_str("8/8/8/8/8/3K1k1p/6p1/5R2 b - - 1 54").unwrap();

        let mut moves1 = Vec::new();
        pawn_moves_front(&position, MoveStage::All, &mut moves1);
        assert_eq_vec_move(&moves1, &["g2g1=Q", "g2g1=R", "g2g1=B", "g2g1=N", "h3h2"]);

        let mut moves2 = Vec::new();
        pawn_moves_front(&position, MoveStage::CapturesAndPromotions, &mut moves2);
        assert_eq_vec_move(&moves2, &["g2g1=Q", "g2g1=R", "g2g1=B", "g2g1=N"]);

        let mut moves3 = Vec::new();
        pawn_moves_front(&position, MoveStage::Quiet, &mut moves3);
        assert_eq_vec_move(&moves3, &["h3h2"]);
    }
}
