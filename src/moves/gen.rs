use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS},
        magics::magic_index,
        magics::BISHOP_MAGICS,
        magics::ROOK_MAGICS,
        specials::generate_king_castles,
        specials::{generate_pawn_moves, pawn_attacks},
    },
    make_move_position, Move, MoveFlag, MoveVec,
};
use crate::{
    moves::make_move_board,
    position::{
        bitboard::Bitboard,
        board::{Board, Piece},
        square::Square,
        Color, Position,
    },
};
use ahash::AHashMap;

pub struct MoveDirection;

impl MoveDirection {
    pub const NORTH: i8 = 8;
    pub const SOUTH: i8 = -8;
    pub const EAST: i8 = 1;
    pub const WEST: i8 = -1;

    pub const fn pawn_direction(color: Color) -> i8 {
        match color {
            Color::White => Self::NORTH,
            Color::Black => Self::SOUTH,
        }
    }
}

pub fn checked_by(board: &Board, color: Color) -> bool {
    let mut checked_king = board.pieces_bb(Piece::King) & board.occupancy_bb(color.opposite());
    checked_king.pop_lsb().map_or(false, |square| square_attacked_by(board, square, color))
}

pub fn square_attacked_by(board: &Board, square: Square, color: Color) -> bool {
    if pawn_attacks(board, color).is_set(square) {
        return true;
    }

    let occupancy = board.occupancy_bb_all();

    let mut super_piece_attacks = (piece_attacks(Piece::Queen, square, occupancy)
        | piece_attacks(Piece::Knight, square, occupancy))
        & board.occupancy_bb(color)
        & !board.pieces_bb(Piece::Pawn);

    while let Some(attacking_square) = super_piece_attacks.pop_lsb() {
        let piece = board.piece_at(attacking_square).unwrap();
        if piece_attacks(piece, attacking_square, occupancy).is_set(square) {
            return true;
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
    let occupancy_us = board.occupancy_bb(color);
    let mut pieces = board.pieces_bb(piece) & occupancy_us;

    if pieces.is_empty() {
        return;
    }

    let occupancy = board.occupancy_bb_all();
    let occupancy_them = board.occupancy_bb(color.opposite());

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

    for piece in Piece::list() {
        match piece {
            Piece::Pawn => generate_pawn_moves::<QUIESCE>(position, &mut moves),
            _ => generate_regular_moves::<QUIESCE>(
                &position.board,
                *piece,
                position.side_to_move,
                &mut moves,
            ),
        }
    }

    if !QUIESCE {
        generate_king_castles(position, &mut moves);
    }

    if !PSEUDO {
        let new_side_to_move = position.side_to_move.opposite();
        moves.retain(|mov| match mov.flag() {
            MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => true,
            _ => {
                let mut new_board = position.board;
                make_move_board(&mut new_board, *mov);
                !checked_by(&new_board, new_side_to_move)
            }
        });
    }

    moves
}

type PerftResult = (u64, Vec<(Move, u64)>);

pub fn perft<const BULK_AT_HORIZON: bool, const HASH: bool, const SILENT: bool>(
    position: &Position,
    depth: u8,
) -> PerftResult {
    perft_internal::<true, BULK_AT_HORIZON, HASH, SILENT>(position, depth, &mut AHashMap::new())
}

fn perft_internal<
    const ROOT: bool,
    const BULK_AT_HORIZON: bool,
    const HASH: bool,
    const SILENT: bool,
>(
    position: &Position,
    depth: u8,
    cache: &mut AHashMap<(Position, u8), PerftResult>,
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
        let new_position = make_move_position(position, mov);
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
        cache.insert((*position, depth), (nodes, res.clone()));
    }

    (nodes, res)
}

#[cfg(test)]
mod tests {
    use crate::position::{
        fen::{KIWIPETE_WHITE_FEN, START_FEN},
        Color, Position,
    };

    use super::perft;

    #[test]
    fn square_is_attacked_1() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert!(super::square_attacked_by(&position.board, super::Square::E4, Color::Black));
        assert!(super::square_attacked_by(&position.board, super::Square::G2, Color::Black));
        assert!(super::square_attacked_by(&position.board, super::Square::A6, Color::White));
        assert!(!super::square_attacked_by(&position.board, super::Square::C7, Color::White));
        assert!(!super::square_attacked_by(&position.board, super::Square::B4, Color::White));
    }

    #[test]
    fn square_is_attacked_2() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        assert!(super::square_attacked_by(&position.board, super::Square::C8, Color::White));
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

    #[test]
    fn perft_simple_start() {
        let (moves, _) = perft::<false, false, true>(&Position::from_fen(START_FEN).unwrap(), 3);
        assert_eq!(moves, 8902);
    }
}
