use crate::position::{
    bitboard::Bitboard,
    square::{Direction, Square},
};

pub static ROOK_MOVE_DIRECTIONS: [Direction; 4] =
    [Square::NORTH, Square::EAST, Square::SOUTH, Square::WEST];

pub static BISHOP_MOVE_DIRECTIONS: [Direction; 4] = [
    Square::NORTH + Square::EAST,
    Square::SOUTH + Square::EAST,
    Square::SOUTH + Square::WEST,
    Square::NORTH + Square::WEST,
];

pub fn slider_attacks_from_square(
    square: Square,
    directions: &[Direction],
    occupancy: Bitboard,
    remove_edges: bool,
) -> Bitboard {
    let mut bb = 0;
    let square = square as Direction;
    let square_file = square % 8;
    let square_rank = square / 8;

    for dir in directions {
        let mut last_file = square_file;
        let mut offset = *dir;

        loop {
            let target_square = square + offset;
            let target_square_file = target_square % 8;
            let target_square_rank = target_square / 8;

            if !(0..64).contains(&target_square) || (target_square_file - last_file).abs() > 2 {
                break;
            }

            let on_edge = (target_square_file == 0 && square_file != 0)
                || (target_square_rank == 0 && square_rank != 0)
                || (target_square_file == 7 && square_file != 7)
                || (target_square_rank == 7 && square_rank != 7);

            if remove_edges && on_edge {
                break;
            }

            bb |= 1 << target_square;

            if on_edge || occupancy.is_set(Square::from(target_square as u8).unwrap()) {
                break;
            }

            offset += *dir;
            last_file = target_square_file;
        }
    }

    Bitboard::new(bb)
}

#[cfg(test)]
mod tests {
    use crate::position::{bitboard::Bitboard, square::Square, Position};
    use std::str::FromStr;

    use super::{slider_attacks_from_square, BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS};

    #[test]
    fn bishop() {
        let position =
            Position::from_str("r3kb1r/3n1p1p/3Bp3/1p1p4/p1qPp3/1NP3Q1/PP3PPP/1K1R3R w kq - 0 18")
                .unwrap();
        assert_eq!(
            slider_attacks_from_square(
                Square::D6,
                &BISHOP_MOVE_DIRECTIONS,
                position.occupancy_bb_all(),
                false,
            ),
            Bitboard::from_square(Square::A3)
                | Bitboard::from_square(Square::A3)
                | Bitboard::from_square(Square::B4)
                | Bitboard::from_square(Square::C5)
                | Bitboard::from_square(Square::E7)
                | Bitboard::from_square(Square::F8)
                | Bitboard::from_square(Square::B8)
                | Bitboard::from_square(Square::C7)
                | Bitboard::from_square(Square::E5)
                | Bitboard::from_square(Square::F4)
                | Bitboard::from_square(Square::G3)
        );
    }

    #[test]
    fn rook() {
        let position =
            Position::from_str("2r1kb1r/3n1p1p/3Bp3/1p1p4/p1qPp3/1NP3QP/PP3PP1/1K1RR3 b k - 0 19")
                .unwrap();

        println!(
            "{}",
            slider_attacks_from_square(
                Square::C8,
                &ROOK_MOVE_DIRECTIONS,
                position.occupancy_bb_all(),
                true,
            )
        );

        assert_eq!(
            slider_attacks_from_square(
                Square::C8,
                &ROOK_MOVE_DIRECTIONS,
                position.occupancy_bb_all(),
                true,
            ),
            Bitboard::from_square(Square::B8)
                | Bitboard::from_square(Square::D8)
                | Bitboard::from_square(Square::E8)
                | Bitboard::from_square(Square::C7)
                | Bitboard::from_square(Square::C6)
                | Bitboard::from_square(Square::C5)
                | Bitboard::from_square(Square::C4)
        );
    }
}
