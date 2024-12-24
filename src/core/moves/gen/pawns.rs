use crate::{
    core::moves::{Move, MoveFlag},
    core::{
        bitboard::Bitboard,
        color::Color,
        piece::Piece,
        square::{Direction, Square},
        MoveStage, Position,
    },
};

use super::leapers::{init_leaper_attacks, LeaperAttackMap};

static PAWN_DIRECTIONS: [Direction; 2] = [Square::NORTH, Square::SOUTH];

pub static PAWN_ATTACKS_WHITE: LeaperAttackMap =
    init_leaper_attacks(&[Square::NORTH + Square::WEST, Square::NORTH + Square::EAST]);

pub static PAWN_ATTACKS_BLACK: LeaperAttackMap =
    init_leaper_attacks(&[Square::SOUTH + Square::WEST, Square::SOUTH + Square::EAST]);

const LAST_RANKS: Bitboard = Bitboard::new(0xFF | ((0xFF) << (8 * 7)));

static DOUBLE_RANKS: [[Bitboard; 3]; 2] = {
    [
        [Bitboard::rank_mask(1), Bitboard::rank_mask(2), Bitboard::rank_mask(3)],
        [Bitboard::rank_mask(6), Bitboard::rank_mask(5), Bitboard::rank_mask(4)],
    ]
};

fn pawn_moves_front(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    let our_pawns = position.pieces_color_bb(Piece::Pawn, position.side_to_move);
    let walk = our_pawns.shifted(PAWN_DIRECTIONS[position.side_to_move() as usize])
        & !position.occupancy_bb_all()
        & match stage {
            MoveStage::HashMove => panic!(),
            MoveStage::All => Bitboard::full(),
            MoveStage::CapturesAndPromotions => LAST_RANKS,
            MoveStage::Quiet => !LAST_RANKS,
        };
    let flipped_direction = PAWN_DIRECTIONS[position.side_to_move().flipped() as usize];

    for sq in walk & LAST_RANKS {
        let from = sq.shifted(flipped_direction);
        moves.push(Move::new(from, sq, MoveFlag::KnightPromotion));
        moves.push(Move::new(from, sq, MoveFlag::BishopPromotion));
        moves.push(Move::new(from, sq, MoveFlag::RookPromotion));
        moves.push(Move::new(from, sq, MoveFlag::QueenPromotion));
    }

    for sq in walk & !LAST_RANKS {
        let from = sq.shifted(flipped_direction);
        moves.push(Move::new(from, sq, MoveFlag::Quiet));
    }
}

fn pawn_moves_double(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    if matches!(stage, MoveStage::CapturesAndPromotions) {
        return;
    }

    let our_pawns = position.pieces_color_bb(Piece::Pawn, position.side_to_move);
    let occupancy = position.occupancy_bb_all();

    let ranks = DOUBLE_RANKS[position.side_to_move() as usize];
    let (second, third, fourth) = (ranks[0], ranks[1], ranks[2]);
    let direction = PAWN_DIRECTIONS[position.side_to_move() as usize];

    let candidates = (our_pawns & second)
        & !(occupancy & third).shifted(-direction)
        & !(occupancy & fourth).shifted(direction * -2);
    for sq in candidates {
        moves.push(Move::new(sq, sq.shifted(direction * 2), MoveFlag::DoublePawnPush));
    }
}

fn pawn_moves_captures(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    if matches!(stage, MoveStage::Quiet) {
        return;
    }

    let our_direction = PAWN_DIRECTIONS[position.side_to_move() as usize];
    let (west_attacks, east_attacks) = pawn_attacks_sided(position, position.side_to_move());
    let ep_bb = position.ep_square().map_or(Bitboard::empty(), Bitboard::from_square);

    for sq in west_attacks & !ep_bb & LAST_RANKS {
        let to = sq.shifted(-our_direction + Square::EAST);
        moves.push(Move::new(to, sq, MoveFlag::KnightPromotionCapture));
        moves.push(Move::new(to, sq, MoveFlag::BishopPromotionCapture));
        moves.push(Move::new(to, sq, MoveFlag::RookPromotionCapture));
        moves.push(Move::new(to, sq, MoveFlag::QueenPromotionCapture));
    }
    for sq in west_attacks & !ep_bb & !LAST_RANKS {
        moves.push(Move::new(sq.shifted(-our_direction + Square::EAST), sq, MoveFlag::Capture));
    }
    for sq in west_attacks & ep_bb {
        moves.push(Move::new(
            sq.shifted(-our_direction + Square::EAST),
            sq,
            MoveFlag::EnpassantCapture,
        ));
    }

    for sq in east_attacks & !ep_bb & LAST_RANKS {
        let to_west = sq.shifted(-our_direction + Square::WEST);
        moves.push(Move::new(to_west, sq, MoveFlag::KnightPromotionCapture));
        moves.push(Move::new(to_west, sq, MoveFlag::BishopPromotionCapture));
        moves.push(Move::new(to_west, sq, MoveFlag::RookPromotionCapture));
        moves.push(Move::new(to_west, sq, MoveFlag::QueenPromotionCapture));
    }
    for sq in east_attacks & !ep_bb & !LAST_RANKS {
        moves.push(Move::new(sq.shifted(-our_direction + Square::WEST), sq, MoveFlag::Capture));
    }
    for sq in east_attacks & ep_bb {
        moves.push(Move::new(
            sq.shifted(-our_direction + Square::WEST),
            sq,
            MoveFlag::EnpassantCapture,
        ));
    }
}

fn pawn_attacks_sided(position: &Position, color: Color) -> (Bitboard, Bitboard) {
    let our_pawns = position.pieces_color_bb(Piece::Pawn, color);
    let their_pieces = position.occupancy_bb(color.flipped())
        | (color == position.side_to_move())
            .then_some(position.ep_square())
            .flatten()
            .map_or(Bitboard::empty(), Bitboard::from_square);
    let our_direction = PAWN_DIRECTIONS[color as usize];

    let west_attacks =
        (our_pawns & !Bitboard::file_mask(0)).shifted(our_direction + Square::WEST) & their_pieces;
    let east_attacks =
        (our_pawns & !Bitboard::file_mask(7)).shifted(our_direction + Square::EAST) & their_pieces;

    (west_attacks, east_attacks)
}

pub fn pawn_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    let attackers = match color.flipped() {
        Color::White => &PAWN_ATTACKS_WHITE,
        Color::Black => &PAWN_ATTACKS_BLACK,
    };
    attackers[square as usize] & position.pieces_color_bb(Piece::Pawn, color)
}

pub fn pawn_moves(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    pawn_moves_front(position, stage, moves);
    pawn_moves_double(position, stage, moves);
    pawn_moves_captures(position, stage, moves);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::core::moves::gen::tests::assert_staged_moves;

    fn pawn_attacks(position: &Position, color: Color) -> Bitboard {
        let (west_attacks, east_attacks) = pawn_attacks_sided(position, color);
        west_attacks | east_attacks
    }

    #[test]
    fn front_1() {
        assert_staged_moves(
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
        assert_staged_moves(
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
        assert_staged_moves(
            "rn1qkb1r/pp3ppp/4pn2/3p2Bb/3P4/2PB1N1P/PP3PP1/RN1QK2R b KQkq - 2 8",
            pawn_moves_double,
            [vec!["b7b5", "a7a5"], vec![], vec!["b7b5", "a7a5"]],
        );
    }

    #[test]
    fn double_2() {
        assert_staged_moves(
            "r3k2r/1pp1qppp/p1p2n2/4b3/4P1b1/2NP1N2/PPP2PPP/R1BQR1K1 w kq - 8 10",
            pawn_moves_double,
            [vec!["a2a4", "b2b4", "h2h4"], vec![], vec!["a2a4", "b2b4", "h2h4"]],
        );
    }

    #[test]
    fn attacks() {
        let position = Position::from_str(
            "3r1rk1/2p1qpp1/p1p4p/Pp2b1B1/4n1b1/2NP1N1P/1PP2PP1/R2QR1K1 w - b6 0 15",
        )
        .unwrap();

        assert_eq!(
            pawn_attacks(&position, Color::White),
            Bitboard::from_square(Square::B6)
                | Bitboard::from_square(Square::E4)
                | Bitboard::from_square(Square::G4)
        );

        assert_eq!(pawn_attacks(&position, Color::Black), Bitboard::from_square(Square::G5));
    }

    #[test]
    fn captures_1() {
        assert_staged_moves(
            "rn1q1rk1/p2b1ppp/3bpn2/3p4/Pp1P4/1BP2N1P/1P1N1PP1/R1BQ1RK1 b - a3 0 11",
            pawn_moves_captures,
            [vec!["b4c3", "b4a3"], vec!["b4c3", "b4a3"], vec![]],
        );
    }

    #[test]
    fn captures_2() {
        assert_staged_moves(
            "4nrk1/1r6/p7/3R1ppp/2P1p1PP/1P3P2/P4B2/3R2K1 w - - 0 28",
            pawn_moves_captures,
            [vec!["h4g5", "g4h5", "g4f5", "f3e4"], vec!["h4g5", "g4h5", "g4f5", "f3e4"], vec![]],
        );
    }

    #[test]
    fn all() {
        assert_staged_moves(
            "rn1q1rk1/p2b1ppp/3bpn2/3p4/Pp1P4/1BP2N1P/1P1N1PP1/R1BQ1RK1 b - a3 0 11",
            pawn_moves,
            [
                vec!["h7h6", "h7h5", "g7g6", "g7g5", "e6e5", "b4c3", "b4a3", "a7a6", "a7a5"],
                vec!["b4c3", "b4a3"],
                vec!["h7h6", "h7h5", "g7g6", "g7g5", "e6e5", "a7a6", "a7a5"],
            ],
        );
    }

    #[test]
    fn attackers() {
        let position = Position::from_str(
            "3r1rk1/2p2pp1/p1p4p/Pp2b1B1/1q2n1b1/2NP1P1P/1PP3PN/R2QR1K1 b - - 0 16",
        )
        .unwrap();

        assert_eq!(
            pawn_attackers(&position, Color::White, Square::B6),
            Bitboard::from_square(Square::A5)
        );

        assert_eq!(
            pawn_attackers(&position, Color::White, Square::E4),
            Bitboard::from_square(Square::F3) | Bitboard::from_square(Square::D3)
        );

        assert_eq!(
            pawn_attackers(&position, Color::Black, Square::C4),
            Bitboard::from_square(Square::B5)
        );

        assert_eq!(pawn_attackers(&position, Color::Black, Square::D4), Bitboard::empty());
    }
}
