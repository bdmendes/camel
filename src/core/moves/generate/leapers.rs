use crate::{
    core::moves::{Move, MoveFlag},
    core::position::{
        Position,
        bitboard::Bitboard,
        color::Color,
        piece::Piece,
        square::{Direction, Square},
    },
};

use super::{MoveStage, MoveVec};

pub type LeaperAttackMap = [Bitboard; 64];

pub const fn init_leaper_attacks(move_directions: &[Direction]) -> LeaperAttackMap {
    let mut attacks: LeaperAttackMap = [Bitboard::empty(); 64];

    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let mut bb = 0;

        let mut i = 0;
        while i < move_directions.len() {
            let target_square = square + move_directions[i];
            let target_square_file = target_square % 8;
            if target_square >= 0 && target_square < 64 && (target_square_file - file).abs() <= 2 {
                bb |= 1 << target_square;
            }
            i += 1;
        }

        attacks[square as usize] = Bitboard::new(bb);
        square += 1
    }

    attacks
}

static KNIGHT_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    Square::NORTH + 2 * Square::WEST,
    Square::NORTH + 2 * Square::EAST,
    Square::SOUTH + 2 * Square::WEST,
    Square::SOUTH + 2 * Square::EAST,
    2 * Square::NORTH + Square::WEST,
    2 * Square::NORTH + Square::EAST,
    2 * Square::SOUTH + Square::WEST,
    2 * Square::SOUTH + Square::EAST,
]);

static KING_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    Square::NORTH,
    Square::NORTH + Square::EAST,
    Square::EAST,
    Square::SOUTH + Square::EAST,
    Square::SOUTH,
    Square::SOUTH + Square::WEST,
    Square::WEST,
    Square::NORTH + Square::WEST,
]);

fn leaper_moves(
    piece: Piece,
    map: &LeaperAttackMap,
    position: &Position,
    stage: MoveStage,
    moves: &mut MoveVec,
) {
    let our_pieces = position.pieces_color_bb(piece, position.side_to_move());
    let ours = position.occupancy_bb(position.side_to_move());
    let theirs = position.occupancy_bb(position.side_to_move().flipped());
    for sq in our_pieces {
        let attacks = map[sq as usize] & !ours;
        if matches!(stage, MoveStage::All | MoveStage::CapturesAndPromotions) {
            (attacks & theirs).for_each(|to| moves.push(Move::new(sq, to, MoveFlag::Capture)));
        }
        if matches!(stage, MoveStage::All | MoveStage::Quiet) {
            (attacks & !theirs).for_each(|to| moves.push(Move::new(sq, to, MoveFlag::Quiet)));
        }
    }
}

pub fn knight_moves(position: &Position, stage: MoveStage, moves: &mut MoveVec) {
    leaper_moves(Piece::Knight, &KNIGHT_ATTACKS, position, stage, moves);
}

pub fn king_regular_moves(position: &Position, stage: MoveStage, moves: &mut MoveVec) {
    leaper_moves(Piece::King, &KING_ATTACKS, position, stage, moves);
}

pub fn knight_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square as usize] & position.pieces_color_bb(Piece::Knight, color)
}

pub fn king_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    KING_ATTACKS[square as usize] & position.pieces_color_bb(Piece::King, color)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{king_attackers, king_regular_moves, knight_attackers, knight_moves};
    use crate::{
        core::moves::generate::tests::assert_staged_moves,
        core::position::{Position, bitboard::Bitboard, color::Color, square::Square},
    };

    #[test]
    fn knights() {
        assert_staged_moves(
            "r1bqkb1r/ppppnppp/2n5/4p3/4P3/N4N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4",
            knight_moves,
            [
                vec![
                    "f3g1", "f3h4", "f3d4", "f3g5", "a3b1", "a3b5", "a3c4", "f3e5",
                ],
                vec!["f3e5"],
                vec!["f3g1", "f3h4", "f3d4", "f3g5", "a3b1", "a3b5", "a3c4"],
            ],
        );
    }

    #[test]
    fn kings() {
        assert_staged_moves(
            "8/8/4p3/3p3p/4p1kP/1K6/5p2/8 b - - 5 50",
            king_regular_moves,
            [
                vec!["g4h3", "g4g3", "g4f3", "g4f4", "g4f5", "g4g5", "g4h4"],
                vec!["g4h4"],
                vec!["g4h3", "g4g3", "g4f3", "g4f4", "g4f5", "g4g5"],
            ],
        );
    }

    #[test]
    fn knight_attack() {
        let position =
            Position::from_str("8/pp3Qpk/7p/3p3K/1nPP1q2/4nNPP/PP6/5R2 w - - 1 34").unwrap();
        assert_eq!(
            knight_attackers(&position, Color::Black, Square::C2),
            Bitboard::from_square(Square::E3) | Bitboard::from_square(Square::B4)
        );
    }

    #[test]
    fn king_attack() {
        let position =
            Position::from_str("8/pp3Qpk/7p/3p3K/1nPP1q2/4nNPP/PP6/5R2 w - - 1 34").unwrap();
        assert_eq!(
            king_attackers(&position, Color::White, Square::G6),
            Bitboard::from_square(Square::H5)
        );
    }
}
