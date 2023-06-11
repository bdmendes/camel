use std::ops::BitOrAssign;

use crate::position::{
    board::{Bitboard, Board, Piece},
    Color,
};

use super::{Move, MoveDirection, MoveFlag};

const KNIGHT_ATTACKS: [Bitboard; 64] = init_knight_attacks();

const fn init_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [0; 64];

    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bb = 0;

        if file >= 1 && rank >= 2 {
            bb |= 1 << (square + MoveDirection::SOUTH * 2 + MoveDirection::WEST);
        }

        if file >= 2 && rank >= 1 {
            bb |= 1 << (square + MoveDirection::SOUTH + MoveDirection::WEST * 2);
        }

        if file <= 5 && rank >= 1 {
            bb |= 1 << (square + MoveDirection::SOUTH + MoveDirection::EAST * 2);
        }

        if file <= 6 && rank >= 2 {
            bb |= 1 << (square + MoveDirection::SOUTH * 2 + MoveDirection::EAST);
        }

        if file <= 6 && rank <= 5 {
            bb |= 1 << (square + MoveDirection::NORTH * 2 + MoveDirection::EAST);
        }

        if file <= 5 && rank <= 6 {
            bb |= 1 << (square + MoveDirection::NORTH + MoveDirection::EAST * 2);
        }

        if file >= 2 && rank <= 6 {
            bb |= 1 << (square + MoveDirection::NORTH + MoveDirection::WEST * 2);
        }

        if file >= 1 && rank <= 5 {
            bb |= 1 << (square + MoveDirection::NORTH * 2 + MoveDirection::WEST);
        }

        attacks[square as usize] = bb;
        square += 1
    }

    attacks
}

pub fn generate_knight_moves(board: &Board, color: Color, moves: &mut Vec<Move>, quiesce: bool) {
    let occupancy_us = board.occupancy_bb(color);
    let occupancy_them = board.occupancy_bb(color.opposite());

    let mut knights = board.pieces_bb(Piece::Knight) & occupancy_us;

    while knights != 0 {
        let from_square = knights.trailing_zeros() as usize;
        knights &= knights - 1;

        let mut attacks = KNIGHT_ATTACKS[from_square] & !occupancy_us;

        while attacks != 0 {
            let to_square = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;

            let flag = if occupancy_them & (1 << to_square) != 0 {
                MoveFlag::Capture
            } else {
                if quiesce {
                    continue;
                }
                MoveFlag::Quiet
            };

            moves.push(Move::new_raw(from_square, to_square, flag));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{fen::KIWIPETE_FEN, square::Square};

    use super::*;

    #[test]
    fn center_clean() {
        let mut board = Board::default();
        let us_color = Color::White;

        board.set_square(Square::E4, Piece::Knight, us_color);

        let mut moves = Vec::new();
        generate_knight_moves(&board, us_color, &mut moves, false);

        assert_eq!(moves.len(), 8);

        let expected_moves = vec![
            Move::new(Square::E4, Square::F2, MoveFlag::Quiet),
            Move::new(Square::E4, Square::G3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::G5, MoveFlag::Quiet),
            Move::new(Square::E4, Square::F6, MoveFlag::Quiet),
            Move::new(Square::E4, Square::D6, MoveFlag::Quiet),
            Move::new(Square::E4, Square::C5, MoveFlag::Quiet),
            Move::new(Square::E4, Square::C3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::D2, MoveFlag::Quiet),
        ];

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn top_left() {
        let mut board = Board::default();
        let us_color = Color::White;

        board.set_square(Square::A8, Piece::Knight, us_color);

        let mut moves = Vec::new();
        generate_knight_moves(&board, us_color, &mut moves, false);

        let expected_moves = vec![
            Move::new(Square::A8, Square::B6, MoveFlag::Quiet),
            Move::new(Square::A8, Square::C7, MoveFlag::Quiet),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn kiwipete_regular() {
        let board = Board::from_fen(KIWIPETE_FEN).unwrap();
        let us_color = Color::White;

        let mut moves = Vec::new();
        generate_knight_moves(&board, us_color, &mut moves, false);

        let expected_moves = [
            Move::new(Square::E5, Square::G4, MoveFlag::Quiet),
            Move::new(Square::E5, Square::G6, MoveFlag::Capture),
            Move::new(Square::E5, Square::F7, MoveFlag::Capture),
            Move::new(Square::E5, Square::D7, MoveFlag::Capture),
            Move::new(Square::E5, Square::C6, MoveFlag::Quiet),
            Move::new(Square::E5, Square::C4, MoveFlag::Quiet),
            Move::new(Square::E5, Square::D3, MoveFlag::Quiet),
            Move::new(Square::C3, Square::B1, MoveFlag::Quiet),
            Move::new(Square::C3, Square::A4, MoveFlag::Quiet),
            Move::new(Square::C3, Square::B5, MoveFlag::Quiet),
            Move::new(Square::C3, Square::D1, MoveFlag::Quiet),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn kiwipete_quiesce() {
        let board = Board::from_fen(KIWIPETE_FEN).unwrap();
        let us_color = Color::White;

        let mut moves = Vec::new();
        generate_knight_moves(&board, us_color, &mut moves, true);

        let expected_moves = [
            Move::new(Square::E5, Square::G6, MoveFlag::Capture),
            Move::new(Square::E5, Square::F7, MoveFlag::Capture),
            Move::new(Square::E5, Square::D7, MoveFlag::Capture),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }
}
