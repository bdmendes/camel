use std::collections::HashMap;

use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS},
        magics::magic_index,
        magics::BISHOP_MAGICS,
        magics::ROOK_MAGICS,
        specials::generate_king_castles,
        specials::{generate_pawn_moves, pawn_attacks},
    },
    make_move, Move, MoveFlag, MoveVec,
};
use crate::position::{
    bitboard::Bitboard,
    board::{Board, Piece, PIECES_NO_PAWN},
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

pub fn color_is_checking(board: &Board, color: Color) -> bool {
    let mut checked_king = board.pieces_bb(Piece::King) & board.occupancy_bb(color.opposite());
    checked_king
        .pop_lsb()
        .map_or(false, |king_square| color_attacks_square(board, king_square, color))
}

pub fn color_attacks_square(board: &Board, square: Square, color: Color) -> bool {
    if pawn_attacks(board, color).is_set(square) {
        return true;
    }

    let occupancy = board.occupancy_bb_all();

    let mut super_piece_attacks = (piece_attacks(Piece::Queen, square, occupancy)
        | piece_attacks(Piece::Knight, square, occupancy))
        & board.occupancy_bb(color);

    while let Some(target_square) = super_piece_attacks.pop_lsb() {
        let piece = board.piece_at(target_square).unwrap().0;
        if piece != Piece::Pawn {
            let target_piece_attacks = piece_attacks(piece, target_square, occupancy);
            if target_piece_attacks.is_set(square) {
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
    moves: &mut MoveVec,
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

pub fn generate_moves<const QUIESCE: bool, const PSEUDO: bool>(position: &Position) -> MoveVec {
    let mut moves = MoveVec::new();

    for piece in PIECES_NO_PAWN.iter() {
        generate_regular_moves::<QUIESCE>(
            &position.board,
            *piece,
            position.side_to_move,
            &mut moves,
        );
    }

    generate_pawn_moves::<QUIESCE>(&position, &mut moves);

    if !QUIESCE {
        generate_king_castles(position, &mut moves);
    }

    if !PSEUDO {
        moves.retain(|mov| match mov.flag() {
            MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => true,
            _ => {
                let new_position = make_move(position, *mov);
                !color_is_checking(&new_position.board, new_position.side_to_move)
            }
        });
    }

    moves
}

pub fn perft<const BULK_AT_HORIZON: bool, const HASH: bool, const SILENT: bool>(
    position: &Position,
    depth: u8,
) -> (u64, Vec<(Move, u64)>) {
    perft_internal::<true, BULK_AT_HORIZON, HASH, SILENT>(position, depth, &mut HashMap::new())
}

fn perft_internal<
    const ROOT: bool,
    const BULK_AT_HORIZON: bool,
    const HASH: bool,
    const SILENT: bool,
>(
    position: &Position,
    depth: u8,
    cache: &mut HashMap<(Position, u8), (u64, Vec<(Move, u64)>)>,
) -> (u64, Vec<(Move, u64)>) {
    if depth == 0 {
        return (1, vec![]);
    }

    if HASH {
        if let Some(res) = cache.get(&(*position, depth)) {
            return res.clone();
        }
    }

    let moves = generate_moves::<false, false>(position);

    if BULK_AT_HORIZON && depth == 1 {
        return (moves.len() as u64, vec![]);
    }

    let mut nodes = 0;
    let mut res = if ROOT { Vec::with_capacity(moves.len()) } else { Vec::new() };

    for mov in moves {
        let new_position = make_move(position, mov);
        let (count, _) =
            perft_internal::<false, BULK_AT_HORIZON, HASH, true>(&new_position, depth - 1, cache);
        nodes += count;

        if ROOT {
            res.push((mov, count));
        }

        if !SILENT {
            println!("{}: {}", mov, count);
        }
    }

    if HASH {
        cache.insert((position.clone(), depth), (nodes, res.clone()));
    }

    (nodes, res)
}

#[cfg(test)]
mod tests {
    use crate::position::{fen::KIWIPETE_WHITE_FEN, Color, Position};

    #[test]
    fn square_is_attacked_1() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert!(super::color_attacks_square(&position.board, super::Square::E4, Color::Black));
        assert!(super::color_attacks_square(&position.board, super::Square::G2, Color::Black));
        assert!(super::color_attacks_square(&position.board, super::Square::A6, Color::White));
        assert!(!super::color_attacks_square(&position.board, super::Square::C7, Color::White));
        assert!(!super::color_attacks_square(&position.board, super::Square::B4, Color::White));
    }

    #[test]
    fn square_is_attacked_2() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        assert!(super::color_attacks_square(&position.board, super::Square::C8, Color::White));
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
