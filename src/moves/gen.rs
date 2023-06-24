use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS},
        magics::magic_index,
        magics::BISHOP_MAGICS,
        magics::ROOK_MAGICS,
        specials::generate_king_castles,
        specials::{generate_pawn_moves, pawn_attacks},
    },
    make_move, Move, MoveFlag,
};
use crate::position::{
    bitboard::Bitboard,
    board::{Board, Piece},
    square::Square,
    Color, Position,
};

pub struct MoveDirection;

impl MoveDirection {
    pub const NORTH: i8 = 8;
    pub const SOUTH: i8 = -8;
    pub const EAST: i8 = 1;
    pub const WEST: i8 = -1;
}

pub fn square_is_attacked(board: &Board, square: Square, color: Color) -> bool {
    if pawn_attacks(board, color).is_set(square) {
        return true;
    }

    let occupancy = board.occupancy_bb_all();
    let occupancy_us = board.occupancy_bb(color);

    for piece in [Piece::Knight, Piece::King, Piece::Rook, Piece::Bishop, Piece::Queen].iter() {
        let mut bb = board.pieces_bb(*piece) & occupancy_us;
        while let Some(from_square) = bb.pop_lsb() {
            if piece_attacks(*piece, from_square, occupancy).is_set(square) {
                return true;
            }
        }
    }

    false
}

pub fn piece_attacks(piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
    match piece {
        Piece::Knight => KNIGHT_ATTACKS[square as usize],
        Piece::King => KING_ATTACKS[square as usize],
        Piece::Rook => {
            let magic = &ROOK_MAGICS[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Bishop => {
            let magic = &BISHOP_MAGICS[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Queen => {
            piece_attacks(Piece::Rook, square, occupancy)
                | piece_attacks(Piece::Bishop, square, occupancy)
        }
        Piece::Pawn => unimplemented!("Pawn attacks are handled separately"),
    }
}

pub fn generate_regular_moves<const QUIESCE: bool>(
    board: &Board,
    piece: Piece,
    color: Color,
    moves: &mut Vec<Move>,
) {
    let occupancy = board.occupancy_bb_all();
    let occupancy_us = board.occupancy_bb(color);
    let occupancy_them = board.occupancy_bb(color.opposite());

    let mut pieces = board.pieces_bb(piece) & occupancy_us;

    while let Some(from_square) = pieces.pop_lsb() {
        let mut attacks = piece_attacks(piece, from_square, occupancy) & !occupancy_us;

        while let Some(to_square) = attacks.pop_lsb() {
            let flag = if occupancy_them.is_set(to_square) {
                MoveFlag::Capture
            } else {
                if QUIESCE {
                    continue;
                }
                MoveFlag::Quiet
            };

            moves.push(Move::new(from_square, to_square, flag));
        }
    }
}

pub fn generate_moves<const QUIESCE: bool, const PSEUDO: bool>(position: &Position) -> Vec<Move> {
    let mut moves = Vec::new();

    for piece in [Piece::Knight, Piece::Bishop, Piece::King, Piece::Queen, Piece::Rook].iter() {
        generate_regular_moves::<QUIESCE>(
            &position.board,
            *piece,
            position.side_to_move,
            moves.as_mut(),
        );
    }

    generate_pawn_moves::<QUIESCE>(&position, moves.as_mut());

    if !QUIESCE {
        generate_king_castles(position, moves.as_mut());
    }

    if !PSEUDO {
        moves.retain(|mov| match mov.flag() {
            MoveFlag::KingsideCastle => match position.side_to_move {
                Color::White => {
                    !square_is_attacked(&position.board, Square::E1, Color::Black)
                        && !square_is_attacked(&position.board, Square::F1, Color::Black)
                        && !square_is_attacked(&position.board, Square::G1, Color::Black)
                }
                Color::Black => {
                    !square_is_attacked(&position.board, Square::E8, Color::White)
                        && !square_is_attacked(&position.board, Square::F8, Color::White)
                        && !square_is_attacked(&position.board, Square::G8, Color::White)
                }
            },
            MoveFlag::QueensideCastle => match position.side_to_move {
                Color::White => {
                    !square_is_attacked(&position.board, Square::E1, Color::Black)
                        && !square_is_attacked(&position.board, Square::D1, Color::Black)
                        && !square_is_attacked(&position.board, Square::C1, Color::Black)
                }
                Color::Black => {
                    !square_is_attacked(&position.board, Square::E8, Color::White)
                        && !square_is_attacked(&position.board, Square::D8, Color::White)
                        && !square_is_attacked(&position.board, Square::C8, Color::White)
                }
            },
            _ => {
                let new_position = make_move(position, *mov);
                let king_square = (new_position.board.pieces_bb(Piece::King)
                    & new_position.board.occupancy_bb(position.side_to_move))
                .pop_lsb()
                .unwrap();
                !square_is_attacked(
                    &new_position.board,
                    king_square,
                    position.side_to_move.opposite(),
                )
            }
        });
    }

    moves
}

#[cfg(test)]
mod tests {
    use crate::position::{fen::KIWIPETE_WHITE_FEN, Color, Position};

    #[test]
    fn square_is_attacked() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert!(super::square_is_attacked(&position.board, super::Square::E4, Color::Black));
        assert!(super::square_is_attacked(&position.board, super::Square::G2, Color::Black));
        assert!(super::square_is_attacked(&position.board, super::Square::A6, Color::White));
        assert!(!super::square_is_attacked(&position.board, super::Square::C7, Color::White));
        assert!(!super::square_is_attacked(&position.board, super::Square::B4, Color::White));
    }

    #[test]
    fn gen_kiwipete_pseudo_regular() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let moves = super::generate_moves::<false, true>(&position);

        assert_eq!(moves.len(), 48);
    }

    #[test]
    fn gen_kiwipete_pseudo_quiesce() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let moves = super::generate_moves::<true, true>(&position);

        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn gen_legals_simple() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let pseudo_moves = super::generate_moves::<false, true>(&position);
        assert_eq!(pseudo_moves.len(), 16);

        let legal_moves = super::generate_moves::<false, false>(&position);
        assert_eq!(legal_moves.len(), 14);
    }

    #[test]
    fn gen_legals_in_check() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        let moves = super::generate_moves::<false, false>(&position);
        assert_eq!(moves.len(), 6);
    }
}
