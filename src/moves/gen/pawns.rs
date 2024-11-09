use ctor::ctor;

use crate::{
    moves::{Move, MoveFlag},
    position::{
        bitboard::Bitboard,
        piece::Piece,
        square::{Direction, Square},
        Position,
    },
};

use super::MoveStage;

static PAWN_DIRECTIONS: [Direction; 2] = [Square::NORTH, Square::SOUTH];

#[ctor]
static LAST_RANKS: Bitboard = Bitboard::rank_mask(0) | Bitboard::rank_mask(7);

#[ctor]
static DOUBLE_RANKS: [[Bitboard; 3]; 2] = {
    [
        [
            Bitboard::rank_mask(1),
            Bitboard::rank_mask(2),
            Bitboard::rank_mask(3),
        ],
        [
            Bitboard::rank_mask(6),
            Bitboard::rank_mask(5),
            Bitboard::rank_mask(4),
        ],
    ]
};

fn pawn_moves_front(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    let our_pawns =
        position.occupancy_bb(position.side_to_move()) & position.pieces_bb(Piece::Pawn);

    let walk = our_pawns.shift(PAWN_DIRECTIONS[position.side_to_move() as usize])
        & match stage {
            MoveStage::All => !position.occupancy_bb_all(),
            MoveStage::CapturesAndPromotions => !position.occupancy_bb_all() & *LAST_RANKS,
            MoveStage::Quiet => !position.occupancy_bb_all() & !(*LAST_RANKS),
        };

    for sq in walk {
        let from = sq.shift(PAWN_DIRECTIONS[position.side_to_move().flipped() as usize]);
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

fn pawn_moves_double(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    if matches!(stage, MoveStage::CapturesAndPromotions) {
        return;
    }

    let our_pawns =
        position.occupancy_bb(position.side_to_move()) & position.pieces_bb(Piece::Pawn);
    let occupancy = position.occupancy_bb_all();

    let ranks = DOUBLE_RANKS[position.side_to_move() as usize];
    let (second, third, fourth) = (ranks[0], ranks[1], ranks[2]);
    let direction = PAWN_DIRECTIONS[position.side_to_move() as usize];

    let candidates = (our_pawns & second)
        & !(occupancy & third).shift(-direction)
        & !(occupancy & fourth).shift(direction * -2);
    for sq in candidates {
        moves.push(Move::new(
            sq,
            sq.shift(direction * 2),
            MoveFlag::DoublePawnPush,
        ));
    }
}

pub fn pawn_moves(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    pawn_moves_front(position, stage, moves);
    pawn_moves_double(position, stage, moves);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::gen::tests::test_staged;

    #[test]
    fn front_1() {
        test_staged(
            "r1bqkbnr/ppp2ppp/2np4/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R w KQkq - 0 4",
            pawn_moves_front,
            [
                vec!["a2a3", "b2b3", "c2c3", "d4d5", "g2g3", "h2h3"],
                vec![],
                vec!["a2a3", "b2b3", "c2c3", "d4d5", "g2g3", "h2h3"],
            ],
        );
    }

    #[test]
    fn front_2() {
        test_staged(
            "8/8/8/8/8/3K1k1p/6p1/5R2 b - - 1 54",
            pawn_moves_front,
            [
                vec!["g2g1=Q", "g2g1=R", "g2g1=B", "g2g1=N", "h3h2"],
                vec!["g2g1=Q", "g2g1=R", "g2g1=B", "g2g1=N"],
                vec!["h3h2"],
            ],
        );
    }

    #[test]
    fn double_1() {
        test_staged(
            "rn1qkb1r/pp3ppp/4pn2/3p2Bb/3P4/2PB1N1P/PP3PP1/RN1QK2R b KQkq - 2 8",
            pawn_moves_double,
            [vec!["b7b5", "a7a5"], vec![], vec!["b7b5", "a7a5"]],
        );
    }

    #[test]
    fn double_2() {
        test_staged(
            "r3k2r/1pp1qppp/p1p2n2/4b3/4P1b1/2NP1N2/PPP2PPP/R1BQR1K1 w kq - 8 10",
            pawn_moves_double,
            [
                vec!["a2a4", "b2b4", "h2h4"],
                vec![],
                vec!["a2a4", "b2b4", "h2h4"],
            ],
        );
    }
}
