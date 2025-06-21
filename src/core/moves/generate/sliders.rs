use crate::{
    core::moves::{Move, MoveFlag},
    core::position::{
        MoveStage, Position,
        bitboard::Bitboard,
        color::Color,
        piece::Piece,
        square::{Direction, Square},
    },
};

use super::{
    MoveVec,
    magics::{bishop_attacks, queen_attacks, rook_attacks},
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

pub fn diagonal_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    let their_bishop_queens = position.occupancy_bb(color)
        & (position.pieces_bb(Piece::Bishop) | position.pieces_bb(Piece::Queen));
    bishop_attacks(position, square) & their_bishop_queens
}

fn slider_moves(
    piece: Piece,
    attacks_fn: fn(&Position, Square) -> Bitboard,
    position: &Position,
    stage: MoveStage,
    moves: &mut MoveVec,
) {
    let our_pieces = position.pieces_color_bb(piece, position.side_to_move());
    let ours = position.occupancy_bb(position.side_to_move());
    let theirs = position.occupancy_bb(position.side_to_move().flipped());
    for sq in our_pieces {
        let attacks = attacks_fn(position, sq) & !ours;
        if matches!(stage, MoveStage::All | MoveStage::CapturesAndPromotions) {
            (attacks & theirs).for_each(|to| moves.push(Move::new(sq, to, MoveFlag::Capture)));
        }
        if matches!(stage, MoveStage::All | MoveStage::Quiet) {
            (attacks & !theirs).for_each(|to| moves.push(Move::new(sq, to, MoveFlag::Quiet)));
        }
    }
}

pub fn file_attackers(position: &Position, color: Color, square: Square) -> Bitboard {
    let their_rook_queens = position.occupancy_bb(color)
        & (position.pieces_bb(Piece::Rook) | position.pieces_bb(Piece::Queen));
    rook_attacks(position, square) & their_rook_queens
}

pub fn rook_moves(position: &Position, stage: MoveStage, moves: &mut MoveVec) {
    slider_moves(Piece::Rook, rook_attacks, position, stage, moves);
}

pub fn bishop_moves(position: &Position, stage: MoveStage, moves: &mut MoveVec) {
    slider_moves(Piece::Bishop, bishop_attacks, position, stage, moves);
}

pub fn queen_moves(position: &Position, stage: MoveStage, moves: &mut MoveVec) {
    slider_moves(Piece::Queen, queen_attacks, position, stage, moves);
}

#[cfg(test)]
mod tests {
    use crate::{
        core::moves::generate::{
            sliders::{diagonal_attackers, file_attackers},
            tests::assert_staged_moves,
        },
        core::position::{Position, bitboard::Bitboard, color::Color, square::Square},
    };
    use std::str::FromStr;

    use super::{
        BISHOP_MOVE_DIRECTIONS, ROOK_MOVE_DIRECTIONS, bishop_moves, queen_moves, rook_moves,
        slider_attacks_from_square,
    };

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

    #[test]
    fn file_attacks() {
        let position = Position::from_str(
            "4k1nr/p1pqppb1/2n3p1/1r1PP2p/1pp2Pb1/5N2/PP2B1PP/RNBQR1K1 w k - 4 13",
        )
        .unwrap();
        assert_eq!(
            file_attackers(&position, Color::Black, Square::D5),
            Bitboard::from_square(Square::D7) | Bitboard::from_square(Square::B5)
        );
    }

    #[test]
    fn diagonal_attacks() {
        let position = Position::from_str(
            "4k1nr/p1q1ppb1/6p1/nrpPP2p/1pp2Pb1/5N2/PP2B1PP/RNBQ1RK1 w k - 2 16",
        )
        .unwrap();
        assert_eq!(
            diagonal_attackers(&position, Color::Black, Square::E5),
            Bitboard::from_square(Square::C7) | Bitboard::from_square(Square::G7)
        );
    }

    #[test]
    fn bishop_move() {
        assert_staged_moves(
            "4k1nr/p1q1ppb1/6p1/nrpPP2p/1pp2Pb1/5N2/PP2B1PP/RNBQ1RK1 w k - 2 16",
            bishop_moves,
            [
                vec!["c1d2", "c1e3", "e2d3", "e2c4"],
                vec!["e2c4"],
                vec!["c1d2", "c1e3", "e2d3"],
            ],
        );
    }

    #[test]
    fn rook_move() {
        assert_staged_moves(
            "4kr2/2q1ppb1/5n2/nppPPpNp/1pp3b1/2N5/1P2B1PP/R1BQ1RK1 w - - 0 21",
            rook_moves,
            [
                vec![
                    "a1b1", "a1a2", "a1a3", "a1a4", "f1e1", "f1f2", "f1f3", "f1f4", "f1f5", "a1a5",
                ],
                vec!["f1f5", "a1a5"],
                vec![
                    "a1b1", "a1a2", "a1a3", "a1a4", "f1e1", "f1f2", "f1f3", "f1f4",
                ],
            ],
        );
    }

    #[test]
    fn queen_move() {
        assert_staged_moves(
            "4kr2/2q1ppb1/3P1n2/npp1PpNp/1pp3b1/2N5/1P2B1PP/R1BQ1RK1 b - - 0 21",
            queen_moves,
            [
                vec![
                    "c7c6", "c7c8", "c7d7", "c7b7", "c7a7", "c7b8", "c7b6", "c7d8", "c7d6",
                ],
                vec!["c7d6"],
                vec![
                    "c7c6", "c7c8", "c7d7", "c7b7", "c7a7", "c7b8", "c7b6", "c7d8",
                ],
            ],
        );
    }
}
