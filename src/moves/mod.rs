use self::gen::MoveDirection;
use crate::position::{
    board::{Board, Piece},
    square::Square,
    CastlingRights, Color, Position,
};
use primitive_enum::primitive_enum;
use smallvec::SmallVec;

pub mod attacks;
pub mod gen;

pub type MoveVec = SmallVec<[Move; 64]>;

primitive_enum! {
    MoveFlag u8;

    // Regular
    Quiet,
    Capture,

    // King specials
    KingsideCastle,
    QueensideCastle,

    // Pawn specials
    DoublePawnPush,
    EnPassantCapture,
    KnightPromotion,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
}

impl MoveFlag {
    pub fn is_reversible(&self) -> bool {
        matches!(self, Self::Quiet)
    }

    pub fn is_quiet(&self) -> bool {
        matches!(
            self,
            Self::Quiet | Self::DoublePawnPush | Self::KingsideCastle | Self::QueensideCastle
        )
    }

    pub fn is_capture(&self) -> bool {
        matches!(
            self,
            Self::Capture
                | Self::EnPassantCapture
                | Self::KnightPromotionCapture
                | Self::BishopPromotionCapture
                | Self::RookPromotionCapture
                | Self::QueenPromotionCapture
        )
    }

    pub fn is_promotion(&self) -> bool {
        matches!(
            self,
            Self::KnightPromotion
                | Self::BishopPromotion
                | Self::RookPromotion
                | Self::QueenPromotion
                | Self::KnightPromotionCapture
                | Self::BishopPromotionCapture
                | Self::RookPromotionCapture
                | Self::QueenPromotionCapture
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    data: u16,
}

impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move { data: (from as u16) | ((to as u16) << 6) | ((flag as u16) << 12) }
    }

    pub fn from(&self) -> Square {
        unsafe { Square::from((self.data & 0b111111) as u8).unwrap_unchecked() }
    }

    pub fn to(&self) -> Square {
        unsafe { Square::from(((self.data >> 6) & 0b111111) as u8).unwrap_unchecked() }
    }

    pub fn flag(&self) -> MoveFlag {
        unsafe { MoveFlag::from((self.data >> 12) as u8).unwrap_unchecked() }
    }

    pub fn promotion_piece(&self) -> Option<Piece> {
        match self.flag() {
            MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Some(Piece::Knight),
            MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Some(Piece::Bishop),
            MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Some(Piece::Rook),
            MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => Some(Piece::Queen),
            _ => None,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str(&self.from().to_string());

        s.push_str(&self.to().to_string());

        match self.flag() {
            MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => s.push('q'),
            MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => s.push('r'),
            MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => s.push('b'),
            MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => s.push('n'),
            _ => (),
        }

        write!(f, "{}", s)
    }
}

pub fn make_move_board(board: &mut Board, mov: Move) {
    let (piece, color) = board.piece_color_at(mov.from()).unwrap();

    board.clear_square(mov.from());

    match mov.flag() {
        MoveFlag::KingsideCastle => match color {
            Color::White => {
                board.clear_square(Square::H1);
                board.set_square::<false>(Square::G1, Piece::King, Color::White);
                board.set_square::<false>(Square::F1, Piece::Rook, Color::White);
            }
            Color::Black => {
                board.clear_square(Square::H8);
                board.set_square::<false>(Square::G8, Piece::King, Color::Black);
                board.set_square::<false>(Square::F8, Piece::Rook, Color::Black);
            }
        },
        MoveFlag::QueensideCastle => match color {
            Color::White => {
                board.clear_square(Square::A1);
                board.set_square::<false>(Square::C1, Piece::King, Color::White);
                board.set_square::<false>(Square::D1, Piece::Rook, Color::White);
            }
            Color::Black => {
                board.clear_square(Square::A8);
                board.set_square::<false>(Square::C8, Piece::King, Color::Black);
                board.set_square::<false>(Square::D8, Piece::Rook, Color::Black);
            }
        },
        MoveFlag::EnPassantCapture => {
            board.set_square::<false>(mov.to(), Piece::Pawn, color);
            board.clear_square(match color {
                Color::White => {
                    Square::from((mov.to() as i8 + MoveDirection::SOUTH) as u8).unwrap()
                }
                Color::Black => {
                    Square::from((mov.to() as i8 + MoveDirection::NORTH) as u8).unwrap()
                }
            });
        }
        MoveFlag::QueenPromotion => {
            board.set_square::<false>(mov.to(), Piece::Queen, color);
        }
        MoveFlag::QueenPromotionCapture => {
            board.set_square::<true>(mov.to(), Piece::Queen, color);
        }
        MoveFlag::RookPromotion => {
            board.set_square::<false>(mov.to(), Piece::Rook, color);
        }
        MoveFlag::RookPromotionCapture => {
            board.set_square::<true>(mov.to(), Piece::Rook, color);
        }
        MoveFlag::BishopPromotion => {
            board.set_square::<false>(mov.to(), Piece::Bishop, color);
        }
        MoveFlag::BishopPromotionCapture => {
            board.set_square::<true>(mov.to(), Piece::Bishop, color);
        }
        MoveFlag::KnightPromotion => {
            board.set_square::<false>(mov.to(), Piece::Knight, color);
        }
        MoveFlag::KnightPromotionCapture => {
            board.set_square::<true>(mov.to(), Piece::Knight, color);
        }
        MoveFlag::Capture => {
            board.set_square::<true>(mov.to(), piece, color);
        }
        _ => {
            board.set_square::<false>(mov.to(), piece, color);
        }
    }
}

pub fn make_move_position(position: &Position, mov: Move) -> Position {
    let piece = position.board.piece_at(mov.from()).unwrap();

    // Update board
    let mut board = position.board;
    make_move_board(&mut board, mov);

    // Update en passant square
    let en_passant_square = match mov.flag() {
        MoveFlag::DoublePawnPush => {
            let direction = -MoveDirection::pawn_direction(position.side_to_move);
            Some(Square::from((mov.to() as i8 + direction) as u8).unwrap())
        }
        _ => None,
    };

    // Update castling rights
    let mut castling_rights = position.castling_rights;
    match mov.flag() {
        MoveFlag::KingsideCastle => match position.side_to_move {
            Color::White => {
                castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
            }
            Color::Black => {
                castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
            }
        },
        MoveFlag::QueensideCastle => match position.side_to_move {
            Color::White => {
                castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
            }
            Color::Black => {
                castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
            }
        },
        MoveFlag::Capture | MoveFlag::Quiet if !castling_rights.is_empty() => {
            match (position.side_to_move, piece, mov.from()) {
                (Color::White, Piece::King, Square::E1) => {
                    castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                (Color::White, Piece::Rook, Square::A1) => {
                    castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                (Color::White, Piece::Rook, Square::H1) => {
                    castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                }
                (Color::Black, Piece::King, Square::E8) => {
                    castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
                (Color::Black, Piece::Rook, Square::A8) => {
                    castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
                (Color::Black, Piece::Rook, Square::H8) => {
                    castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                }
                _ => (),
            }
        }
        _ => {}
    }

    // Update side to move
    let side_to_move = position.side_to_move.opposite();

    // Update halfmove clock
    let halfmove_clock = if piece == Piece::Pawn || mov.flag().is_capture() {
        0
    } else {
        position.halfmove_clock + 1
    };

    // Update fullmove number
    let fullmove_number = if position.side_to_move == Color::Black {
        position.fullmove_number + 1
    } else {
        position.fullmove_number
    };

    Position {
        board,
        side_to_move,
        castling_rights,
        en_passant_square,
        halfmove_clock,
        fullmove_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{
        fen::{KIWIPETE_BLACK_FEN, KIWIPETE_WHITE_FEN},
        square::Square,
    };

    #[test]
    fn unpack_move() {
        let m = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        assert_eq!(m.from(), Square::E2);
        assert_eq!(m.to(), Square::E4);
        assert_eq!(m.flag(), MoveFlag::DoublePawnPush);
    }

    #[test]
    fn write_move() {
        let m = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        assert_eq!(m.to_string(), "e2e4");

        let m = Move::new(Square::E7, Square::E8, MoveFlag::QueenPromotion);
        assert_eq!(m.to_string(), "e7e8q");
    }

    #[test]
    fn make_move_simple() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let new_position = super::make_move_position(
            &position,
            Move::new(Square::D5, Square::E6, MoveFlag::Capture),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p1ppqpb1/bn2Pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1"
        );
    }

    #[test]
    fn make_move_castle_long() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let new_position = super::make_move_position(
            &position,
            Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R b kq - 1 1"
        );
    }

    #[test]
    fn make_move_castle_short() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let new_position = super::make_move_position(
            &position,
            Move::new(Square::E1, Square::G1, MoveFlag::KingsideCastle),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R4RK1 b kq - 1 1"
        );
    }

    #[test]
    fn make_move_double_pawn_push() {
        let position = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();

        let new_position = super::make_move_position(
            &position,
            Move::new(Square::C7, Square::C5, MoveFlag::DoublePawnPush),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq c6 0 2"
        );

        let new_position = super::make_move_position(
            &new_position,
            Move::new(Square::D5, Square::C6, MoveFlag::EnPassantCapture),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p2pqpb1/bnP1pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 2"
        );
    }

    #[test]
    fn make_move_promotion() {
        let mut position = Position::from_fen(KIWIPETE_BLACK_FEN).unwrap();

        for mov in &[
            Move::new(Square::H3, Square::G2, MoveFlag::Capture),
            Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle),
            Move::new(Square::G2, Square::H1, MoveFlag::QueenPromotionCapture),
        ] {
            position = super::make_move_position(&position, *mov);
        }

        assert_eq!(
            position.to_fen(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBP1P/2KR3q w kq - 0 3"
        );
    }
}
