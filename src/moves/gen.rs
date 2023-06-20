use crate::position::{
    bitboard::Bitboard,
    board::{Board, Piece},
    square::Square,
    Color, Position,
};

use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS},
        magics::magic_index,
        magics::BISHOP_MAGICS,
        magics::ROOK_MAGICS,
        specials::generate_king_castles,
        specials::generate_pawn_moves,
    },
    Move, MoveFlag,
};

pub struct MoveDirection;

impl MoveDirection {
    pub const NORTH: i8 = 8;
    pub const SOUTH: i8 = -8;
    pub const EAST: i8 = 1;
    pub const WEST: i8 = -1;
}

pub fn piece_attacks(piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
    match piece {
        Piece::Knight => KNIGHT_ATTACKS[square as usize],
        Piece::King => KING_ATTACKS[square as usize],
        Piece::Rook => {
            let magic = &ROOK_MAGICS.get().unwrap()[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Bishop => {
            let magic = &BISHOP_MAGICS.get().unwrap()[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Queen => {
            piece_attacks(Piece::Rook, square, occupancy)
                | piece_attacks(Piece::Bishop, square, occupancy)
        }
        Piece::Pawn => todo!(),
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
        moves.retain(|m| true); // TODO: check if move is legal (king attacked or castle and king passent)
    }

    moves
}
