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

    pub const fn pawn_direction(color: Color) -> i8 {
        match color {
            Color::White => Self::NORTH,
            Color::Black => Self::SOUTH,
        }
    }
}

pub fn checked_by(board: &Board, color: Color) -> bool {
    let mut checked_king = board.pieces_bb_color(Piece::King, color.opposite());
    checked_king.next().map_or(false, |king_square| square_attacked_by(board, king_square, color))
}

pub fn square_attacked_by(board: &Board, square: Square, color: Color) -> bool {
    // Attacked by pawn
    if pawn_attacks(board, color).is_set(square) {
        return true;
    }

    let occupancy_attacker = board.occupancy_bb(color);

    // Attacked by knight
    let attacker_knights = board.pieces_bb(Piece::Knight) & occupancy_attacker;
    let knight_attacks = KNIGHT_ATTACKS[square as usize];
    if (knight_attacks & attacker_knights).is_not_empty() {
        return true;
    }

    // Attacked by king
    let attacker_kings = board.pieces_bb(Piece::King) & occupancy_attacker;
    let king_attacks = KING_ATTACKS[square as usize];
    if (king_attacks & attacker_kings).is_not_empty() {
        return true;
    }

    let occupancy = board.occupancy_bb_all();

    // Attacked in file or rank
    let attacker_rooks_queens =
        (board.pieces_bb(Piece::Rook) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let rook_attacks = piece_attacks(Piece::Rook, square, occupancy);
    if (rook_attacks & attacker_rooks_queens).is_not_empty() {
        return true;
    }

    // Attacked in diagonal
    let attacker_bishops_queens =
        (board.pieces_bb(Piece::Bishop) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let bishop_attacks = piece_attacks(Piece::Bishop, square, occupancy);
    if (bishop_attacks & attacker_bishops_queens).is_not_empty() {
        return true;
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

    for from_square in board.pieces_bb_color(piece, color) {
        let attacks = if QUIESCE {
            piece_attacks(piece, from_square, occupancy) & occupancy_them
        } else {
            piece_attacks(piece, from_square, occupancy) & !occupancy_us
        };

        for to_square in attacks {
            moves.push(Move::new(
                from_square,
                to_square,
                if occupancy_them.is_set(to_square) { MoveFlag::Capture } else { MoveFlag::Quiet },
            ));
        }
    }
}

pub fn generate_moves<const QUIESCE: bool>(position: &Position) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    let board = &position.board;
    let side_to_move = position.side_to_move;

    generate_pawn_moves::<QUIESCE>(position, &mut moves);
    generate_regular_moves::<QUIESCE>(board, Piece::Queen, side_to_move, &mut moves);
    generate_regular_moves::<QUIESCE>(board, Piece::Rook, side_to_move, &mut moves);
    generate_regular_moves::<QUIESCE>(board, Piece::Bishop, side_to_move, &mut moves);
    generate_regular_moves::<QUIESCE>(board, Piece::Knight, side_to_move, &mut moves);
    generate_regular_moves::<QUIESCE>(board, Piece::King, side_to_move, &mut moves);

    if !QUIESCE {
        generate_king_castles(position, &mut moves);
    }

    let is_check = checked_by(board, side_to_move.opposite());
    let king_square = board.pieces_bb_color(Piece::King, side_to_move).next().unwrap();
    let king_rays = piece_attacks(Piece::Queen, king_square, board.occupancy_bb_all());
    let possibly_pinned = king_rays & position.board.occupancy_bb(side_to_move);

    moves.retain(|mov| match mov.flag() {
        MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => {
            // Already validated by the castle generator
            true
        }
        MoveFlag::EnPassantCapture => {
            // Enpassant is a complex case. We must resort to full move making
            let new_position = make_move::<false>(position, *mov);
            !checked_by(&new_position.board, new_position.side_to_move)
        }
        _ if mov.from() != king_square => {
            if !is_check && !possibly_pinned.is_set(mov.from()) {
                // We are moving a piece not in the kings ray, so we can't end up in check
                return true;
            }

            if is_check
                && !board.pieces_bb(Piece::Knight).is_set(mov.to())
                && !king_rays.is_set(mov.to())
            {
                // We are in check and not attempting to block or capture the checking piece
                return false;
            }

            let mut new_board = *board;
            new_board.clear_square(mov.from());
            new_board.set_square::<true>(mov.to(), Piece::Pawn, side_to_move);
            !checked_by(&new_board, side_to_move.opposite())
        }
        _ => {
            let mut new_board = *board;
            new_board.clear_square(king_square);
            new_board.set_square::<true>(mov.to(), Piece::King, side_to_move);
            !checked_by(&new_board, side_to_move.opposite())
        }
    });

    moves
}

pub fn perft(position: &Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_moves::<false>(position);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;

    for mov in moves {
        let new_position = make_move::<true>(position, mov);
        let count = perft(&new_position, depth - 1);
        nodes += count;
    }

    nodes
}

#[cfg(test)]
mod tests {
    use crate::position::{
        fen::{FromFen, KIWIPETE_WHITE_FEN},
        Color, Position,
    };

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
    fn gen_simple() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let moves = super::generate_moves::<false>(&position);
        assert_eq!(moves.len(), 14);
    }

    #[test]
    fn gen_in_check() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        let moves = super::generate_moves::<false>(&position);
        assert_eq!(moves.len(), 6);
    }
}
