use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS_BLACK, PAWN_ATTACKS_WHITE},
        magics::{magic_index, BISHOP_MAGICS, ROOK_MAGICS},
        specials::{generate_king_castles, generate_pawn_moves},
    },
    make_move, Move, MoveFlag,
};
use crate::position::{
    bitboard::Bitboard,
    board::{Board, Piece},
    square::Square,
    Color, Position,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveStage {
    HashMove,
    CapturesAndPromotions,
    NonCaptures,
    All,
}

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

pub fn king_square_attackers<const EARLY_RETURN: bool>(board: &Board, color: Color) -> Bitboard {
    let checked_king = board.pieces_bb_color(Piece::King, color.opposite());
    square_attackers::<EARLY_RETURN>(board, checked_king.into_iter().next().unwrap(), color)
}

pub fn square_attackers<const EARLY_RETURN: bool>(
    board: &Board,
    square: Square,
    color: Color,
) -> Bitboard {
    let mut bb = Bitboard::new(0);
    let occupancy = board.occupancy_bb_all();
    let occupancy_attacker = board.occupancy_bb(color);

    // Attacked in file or rank
    let attacker_rooks_queens =
        (board.pieces_bb(Piece::Rook) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let rook_attacks = piece_attacks(Piece::Rook, square, occupancy, color);
    bb |= rook_attacks & attacker_rooks_queens;

    if EARLY_RETURN && bb.is_not_empty() {
        return bb;
    }

    // Attacked in diagonal
    let attacker_bishops_queens =
        (board.pieces_bb(Piece::Bishop) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let bishop_attacks = piece_attacks(Piece::Bishop, square, occupancy, color);
    bb |= bishop_attacks & attacker_bishops_queens;

    if EARLY_RETURN && bb.is_not_empty() {
        return bb;
    }

    // Attacked by pawn
    bb |= match color {
        Color::White => PAWN_ATTACKS_BLACK[square as usize],
        Color::Black => PAWN_ATTACKS_WHITE[square as usize],
    } & board.pieces_bb(Piece::Pawn)
        & occupancy_attacker;

    if EARLY_RETURN && bb.is_not_empty() {
        return bb;
    }

    // Attacked by knight
    let attacker_knights = board.pieces_bb(Piece::Knight) & occupancy_attacker;
    let knight_attacks = KNIGHT_ATTACKS[square as usize];
    bb |= knight_attacks & attacker_knights;

    if EARLY_RETURN && bb.is_not_empty() {
        return bb;
    }

    // Attacked by king
    let attacker_kings = board.pieces_bb(Piece::King) & occupancy_attacker;
    let king_attacks = KING_ATTACKS[square as usize];
    bb |= king_attacks & attacker_kings;

    bb
}

pub fn piece_attacks(piece: Piece, square: Square, occupancy: Bitboard, color: Color) -> Bitboard {
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
            piece_attacks(Piece::Rook, square, occupancy, color)
                | piece_attacks(Piece::Bishop, square, occupancy, color)
        }
        Piece::Pawn => match color {
            Color::White => PAWN_ATTACKS_WHITE[square as usize],
            Color::Black => PAWN_ATTACKS_BLACK[square as usize],
        },
    }
}

pub fn generate_regular_moves(
    stage: MoveStage,
    board: &Board,
    piece: Piece,
    color: Color,
    moves: &mut Vec<Move>,
) {
    let occupancy = board.occupancy_bb_all();
    let occupancy_us = board.occupancy_bb(color);
    let occupancy_them = board.occupancy_bb(color.opposite());

    for from_square in board.pieces_bb_color(piece, color) {
        let attacks = match stage {
            MoveStage::HashMove => panic!("Hash move should not be generated here"),
            MoveStage::CapturesAndPromotions => {
                piece_attacks(piece, from_square, occupancy, color) & occupancy_them
            }
            MoveStage::NonCaptures => {
                piece_attacks(piece, from_square, occupancy, color) & !occupancy
            }
            MoveStage::All => piece_attacks(piece, from_square, occupancy, color) & !occupancy_us,
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

pub fn generate_moves(stage: MoveStage, position: &Position) -> Vec<Move> {
    let mut moves = Vec::with_capacity(64);
    let board = &position.board;
    let side_to_move = position.side_to_move;

    let checkers = king_square_attackers::<false>(board, side_to_move.opposite());

    if checkers.count_ones() > 1 {
        // Double check requires the king to move.
        generate_regular_moves(stage, board, Piece::King, side_to_move, &mut moves);
    } else {
        generate_pawn_moves(stage, position, &mut moves);
        generate_regular_moves(stage, board, Piece::Queen, side_to_move, &mut moves);
        generate_regular_moves(stage, board, Piece::Rook, side_to_move, &mut moves);
        generate_regular_moves(stage, board, Piece::Bishop, side_to_move, &mut moves);
        generate_regular_moves(stage, board, Piece::Knight, side_to_move, &mut moves);
        generate_regular_moves(stage, board, Piece::King, side_to_move, &mut moves);

        // We can't castle in check.
        if checkers.is_empty() && matches!(stage, MoveStage::All | MoveStage::NonCaptures) {
            generate_king_castles(position, &mut moves);
        }
    }

    let king_square = board.pieces_bb_color(Piece::King, side_to_move).next().unwrap();

    moves.retain(|mov| {
        match mov.flag() {
            MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => {
                // Already validated by the castle generator.
                return true;
            }
            MoveFlag::EnPassantCapture => {
                // Enpassant is too "wild" to deduce rules, so resort to full move making.
                let new_position = make_move(position, *mov);
                return king_square_attackers::<true>(&new_position.board, side_to_move.opposite())
                    .is_empty();
            }
            _ if mov.from() != king_square => {
                let king_rays = piece_attacks(
                    Piece::Queen,
                    king_square,
                    board.occupancy_bb_all(),
                    side_to_move,
                );
                let possibly_pinned = king_rays & position.board.occupancy_bb(side_to_move);

                match checkers.count_ones() {
                    1 => {
                        if checkers.is_set(mov.to()) && !possibly_pinned.is_set(mov.from()) {
                            // We captured the checking piece.
                            return true;
                        }

                        if !board.pieces_bb(Piece::Knight).is_set(mov.to())
                            && !king_rays.is_set(mov.to())
                        {
                            // We are not attempting to block the check.
                            return false;
                        }
                    }
                    0 => {
                        if !possibly_pinned.is_set(mov.from()) {
                            // We are moving a piece not in the kings ray, so we can't end up in check
                            return true;
                        }
                    }
                    _ => panic!("We can't move pieces other than the king when in double check."),
                }
            }
            _ => {}
        }

        let mut new_board = *board;
        new_board.clear_square(mov.from());
        new_board.set_square(
            mov.to(),
            if king_square == mov.from() { Piece::King } else { Piece::Pawn },
            side_to_move,
        );
        king_square_attackers::<true>(&new_board, side_to_move.opposite()).is_empty()
    });

    moves
}

pub fn perft<const STAGED: bool, const ROOT: bool, const LEGALITY_TEST: bool>(
    position: &Position,
    depth: u8,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = if STAGED {
        let mut moves = generate_moves(MoveStage::CapturesAndPromotions, position);
        moves.append(&mut generate_moves(MoveStage::NonCaptures, position));
        moves
    } else {
        generate_moves(MoveStage::All, position)
    };

    if LEGALITY_TEST {
        for mov in &moves {
            assert!(mov.is_pseudo_legal(position));
        }
    }

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;

    for mov in moves {
        let new_position = make_move(position, mov);
        let count = perft::<STAGED, false, LEGALITY_TEST>(&new_position, depth - 1);
        nodes += count;

        if ROOT {
            println!("{}: {}", mov, count);
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::gen::MoveStage,
        position::{
            bitboard::Bitboard,
            fen::{FromFen, KIWIPETE_WHITE_FEN},
            square::Square,
            Color, Position,
        },
    };

    #[test]
    fn square_attackers_1() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::E4, Color::Black),
            Bitboard::new(1 << Square::F6 as usize)
        );
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::G2, Color::Black),
            Bitboard::new(1 << Square::H3 as usize)
        );
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::A6, Color::White),
            Bitboard::new(1 << Square::E2 as usize)
        );
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::D5, Color::Black),
            Bitboard::new(
                1 << Square::E6 as usize | 1 << Square::F6 as usize | 1 << Square::B6 as usize
            )
        );
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::C7, Color::White),
            Bitboard::new(0)
        );
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::B4, Color::White),
            Bitboard::new(0)
        );
    }

    #[test]
    fn square_attackers_2() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::C8, Color::White),
            Bitboard::new(1 << Square::B7 as usize)
        );
    }

    #[test]
    fn square_attackers_3() {
        let position = Position::from_fen("K1k5/1p4p1/8/8/8/1q6/8/8 w - - 100 128").unwrap();
        assert_eq!(
            super::square_attackers::<false>(&position.board, super::Square::A8, Color::Black),
            Bitboard::new(0)
        );
    }

    #[test]
    fn gen_simple_all() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let moves = super::generate_moves(MoveStage::All, &position);
        assert_eq!(moves.len(), 14);
    }

    #[test]
    fn gen_simple_captures() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let moves = super::generate_moves(MoveStage::CapturesAndPromotions, &position);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn gen_simple_non_captures() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let moves = super::generate_moves(MoveStage::NonCaptures, &position);
        assert_eq!(moves.len(), 13);
    }

    #[test]
    fn gen_in_check_all() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        let moves = super::generate_moves(MoveStage::All, &position);
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn gen_in_check_captures() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        let moves = super::generate_moves(MoveStage::CapturesAndPromotions, &position);
        assert_eq!(moves.len(), 0);
    }
}
