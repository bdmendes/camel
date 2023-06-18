use crate::{
    moves::gen::AttackMap,
    position::{bitboard::Bitboard, square::Square},
};

type RookAttackMap = [Bitboard; 4096 * 64];

const ROOK_BLOCKERS_MASK: AttackMap = init_rook_blockers_mask();

const fn rook_attacks_from_square<const REMOVE_CORNERS: bool>(
    square: i32,
    occupancy: Option<Bitboard>,
) -> Bitboard {
    let file = square % 8;
    let rank = square / 8;
    let mut bb = 0;
    let ocuppancy_raw: u64 = match occupancy {
        Some(occupancy) => occupancy.raw(),
        None => 0,
    };
    let edge_offset = if REMOVE_CORNERS { 1 } else { 0 };

    // E
    let mut i = 1;
    while file + i < 8 - edge_offset {
        bb |= 1 << (square + i);
        if (ocuppancy_raw & (1 << (square + i))) != 0 {
            break;
        }
        i += 1;
    }

    // W
    i = 1;
    while file - i >= 0 + edge_offset {
        bb |= 1 << (square - i);
        if (ocuppancy_raw & (1 << (square - i))) != 0 {
            break;
        }
        i += 1;
    }

    // N
    i = 1;
    while rank + i < 8 - edge_offset {
        bb |= 1 << (square + i * 8);
        if (ocuppancy_raw & (1 << (square + i * 8))) != 0 {
            break;
        }
        i += 1;
    }

    // S
    i = 1;
    while rank - i >= 0 + edge_offset {
        bb |= 1 << (square - i * 8);
        if (ocuppancy_raw & (1 << (square - i * 8))) != 0 {
            break;
        }
        i += 1;
    }

    Bitboard::new(bb)
}

const fn init_rook_blockers_mask() -> AttackMap {
    let mut blockers_mask: AttackMap = [Bitboard::new(0); 64];

    let mut square = 0;
    while square < 64 {
        blockers_mask[square as usize] = rook_attacks_from_square::<true>(square, None);
        square += 1;
    }

    blockers_mask
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manual_rook_moves(square: Square, occupancy: Option<Bitboard>) -> Bitboard {
        rook_attacks_from_square::<false>(square as i32, occupancy)
    }

    #[test]
    fn rook_on_center() {
        let square = Square::E4;
        let occupancy = None;

        let expected_squares = [
            Square::E5,
            Square::E6,
            Square::E7,
            Square::E8,
            Square::E3,
            Square::E2,
            Square::E1,
            Square::D4,
            Square::C4,
            Square::B4,
            Square::A4,
            Square::F4,
            Square::G4,
            Square::H4,
        ];

        let mut rook_atacks = manual_rook_moves(square, occupancy);

        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            println!("{}", square);
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }

    #[test]
    fn rook_on_center_blocked() {
        let square = Square::E4;
        let mut occupancy = Bitboard::new(0);
        occupancy.set(Square::E7);
        occupancy.set(Square::E1);
        occupancy.set(Square::D4);
        occupancy.set(Square::G4);

        let expected_squares = [
            Square::E5,
            Square::E6,
            Square::E7,
            Square::E3,
            Square::E2,
            Square::E1,
            Square::D4,
            Square::F4,
            Square::G4,
        ];

        let mut rook_atacks = manual_rook_moves(square, Some(occupancy));

        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            println!("{}", square);
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }

    #[test]
    fn rook_on_center_mask() {
        let square = Square::E4;

        let expected_squares = [
            Square::E5,
            Square::E6,
            Square::E7,
            Square::E3,
            Square::E2,
            Square::D4,
            Square::C4,
            Square::B4,
            Square::F4,
            Square::G4,
        ];

        let mut rook_atacks = ROOK_BLOCKERS_MASK[square as usize];
        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            println!("{}", square);
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }
}
