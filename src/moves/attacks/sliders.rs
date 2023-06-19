use crate::{moves::gen::MoveDirection, position::bitboard::Bitboard};

pub type BlockersMask = [Bitboard; 64];

pub const ROOK_MOVE_DIRECTIONS: [i8; 4] =
    [MoveDirection::NORTH, MoveDirection::EAST, MoveDirection::SOUTH, MoveDirection::WEST];

pub const BISHOP_MOVE_DIRECTIONS: [i8; 4] = [
    MoveDirection::NORTH + MoveDirection::EAST,
    MoveDirection::SOUTH + MoveDirection::EAST,
    MoveDirection::SOUTH + MoveDirection::WEST,
    MoveDirection::NORTH + MoveDirection::WEST,
];

pub const fn slider_attacks_from_square<const REMOVE_EDGES: bool>(
    square: i8,
    move_directions: &[i8],
    occupancy: Option<Bitboard>,
) -> Bitboard {
    let mut bb = 0;
    let ocuppancy_raw: u64 = match occupancy {
        Some(occupancy) => occupancy.raw(),
        None => 0,
    };
    let square_file = square % 8;
    let square_rank = square / 8;

    let mut i = 0;
    while i < move_directions.len() {
        let mut multiplier = 1;
        loop {
            let target_square = square + move_directions[i] * multiplier;
            let target_square_file = target_square % 8;
            let target_square_rank = target_square / 8;

            if target_square < 0 || target_square >= 64 {
                break;
            }

            let on_edge = (target_square_file == 0 && square_file != 0)
                || (target_square_rank == 0 && square_rank != 0)
                || (target_square_file == 7 && square_file != 7)
                || (target_square_rank == 7 && square_rank != 7);

            if REMOVE_EDGES && on_edge {
                break;
            }

            bb |= 1 << target_square;

            if on_edge || ((ocuppancy_raw & (1 << target_square)) != 0) {
                break;
            }

            multiplier += 1;
        }

        i += 1;
    }

    Bitboard::new(bb)
}

pub const fn init_slider_blockers_mask(move_directions: &[i8]) -> BlockersMask {
    let mut blockers_mask: BlockersMask = [Bitboard::new(0); 64];

    let mut square = 0;
    while square < 64 {
        blockers_mask[square as usize] =
            slider_attacks_from_square::<true>(square, move_directions, None);
        square += 1;
    }

    blockers_mask
}

#[cfg(test)]
mod tests {
    use crate::position::square::Square;

    use super::*;

    fn rook_moves(square: Square, occupancy: Option<Bitboard>) -> Bitboard {
        slider_attacks_from_square::<false>(square as i8, &ROOK_MOVE_DIRECTIONS, occupancy)
    }

    fn bishop_moves(square: Square, occupancy: Option<Bitboard>) -> Bitboard {
        slider_attacks_from_square::<false>(square as i8, &BISHOP_MOVE_DIRECTIONS, occupancy)
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

        let mut rook_atacks = rook_moves(square, occupancy);

        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
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

        let mut rook_atacks = rook_moves(square, Some(occupancy));

        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }

    #[test]
    fn rook_on_center_mask() {
        let square = Square::E4;
        let blockers_mask = init_slider_blockers_mask(&ROOK_MOVE_DIRECTIONS);

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

        let mut rook_atacks = blockers_mask[square as usize];
        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }

    #[test]
    fn rook_on_corner_mask() {
        let square = Square::B1;
        let blockers_mask = init_slider_blockers_mask(&ROOK_MOVE_DIRECTIONS);

        let expected_squares = [
            Square::B2,
            Square::B3,
            Square::B4,
            Square::B5,
            Square::B6,
            Square::B7,
            Square::C1,
            Square::D1,
            Square::E1,
            Square::F1,
            Square::G1,
        ];

        let mut rook_atacks = blockers_mask[square as usize];
        let mut found_count = 0;
        while let Some(square) = rook_atacks.pop_lsb() {
            println!("{}", square);
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }

    #[test]
    fn bishop_on_center() {
        let square = Square::E4;
        let occupancy = None;

        let expected_squares = [
            Square::F5,
            Square::G6,
            Square::H7,
            Square::D5,
            Square::C6,
            Square::B7,
            Square::A8,
            Square::F3,
            Square::G2,
            Square::H1,
            Square::D3,
            Square::C2,
            Square::B1,
        ];

        let mut bishop_atacks = bishop_moves(square, occupancy);

        let mut found_count = 0;
        while let Some(square) = bishop_atacks.pop_lsb() {
            assert!(expected_squares.contains(&square));
            found_count += 1;
        }

        assert_eq!(found_count, expected_squares.len());
    }
}
