use self::gen::MoveDirection;
use crate::position::{board::Piece, square::Square, CastlingRights, Color, Position};
use primitive_enum::primitive_enum;

pub mod attacks;
pub mod gen;

primitive_enum!(
    MoveFlag u8;

    // Regular
    Quiet = 0b0000,
    Capture = 0b0001,

    // King specials
    KingsideCastle = 0b0010,
    QueensideCastle = 0b0011,

    // Pawn specials
    DoublePawnPush = 0b0100,
    EnPassantCapture = 0b0101,
    KnightPromotion = 0b0110,
    BishopPromotion = 0b0111,
    RookPromotion = 0b1000,
    QueenPromotion = 0b1001,
    KnightPromotionCapture = 0b1010,
    BishopPromotionCapture = 0b1011,
    RookPromotionCapture = 0b1100,
    QueenPromotionCapture = 0b1101,
);

impl MoveFlag {
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
        Square::from((self.data & 0b111111) as u8).unwrap()
    }

    pub fn to(&self) -> Square {
        Square::from(((self.data >> 6) & 0b111111) as u8).unwrap()
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from((self.data >> 12) as u8).unwrap()
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

pub fn make_move<const UPDATE_METADATA: bool>(position: &Position, mov: Move) -> Position {
    let mut new_board = position.board;
    let mut new_castling_rights = position.castling_rights;
    let mut new_en_passant_square = None;

    let piece = new_board.piece_at(mov.from()).unwrap();

    new_board.clear_square(mov.from());

    // Make move on the board
    match mov.flag() {
        MoveFlag::KingsideCastle => match position.side_to_move {
            Color::White => {
                new_board.clear_square(Square::H1);
                new_board.set_square::<false>(Square::G1, Piece::King, Color::White);
                new_board.set_square::<false>(Square::F1, Piece::Rook, Color::White);
                if UPDATE_METADATA {
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
            }
            Color::Black => {
                new_board.clear_square(Square::H8);
                new_board.set_square::<false>(Square::G8, Piece::King, Color::Black);
                new_board.set_square::<false>(Square::F8, Piece::Rook, Color::Black);
                if UPDATE_METADATA {
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
            }
        },
        MoveFlag::QueensideCastle => match position.side_to_move {
            Color::White => {
                new_board.clear_square(Square::A1);
                new_board.set_square::<false>(Square::C1, Piece::King, Color::White);
                new_board.set_square::<false>(Square::D1, Piece::Rook, Color::White);
                if UPDATE_METADATA {
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
            }
            Color::Black => {
                new_board.clear_square(Square::A8);
                new_board.set_square::<false>(Square::C8, Piece::King, Color::Black);
                new_board.set_square::<false>(Square::D8, Piece::Rook, Color::Black);
                if UPDATE_METADATA {
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
            }
        },
        MoveFlag::EnPassantCapture => {
            new_board.set_square::<false>(mov.to(), Piece::Pawn, position.side_to_move);
            new_board.clear_square(match position.side_to_move {
                Color::White => {
                    Square::from((mov.to() as i8 + MoveDirection::SOUTH) as u8).unwrap()
                }
                Color::Black => {
                    Square::from((mov.to() as i8 + MoveDirection::NORTH) as u8).unwrap()
                }
            });
        }
        MoveFlag::QueenPromotion => {
            new_board.set_square::<false>(mov.to(), Piece::Queen, position.side_to_move);
        }
        MoveFlag::QueenPromotionCapture => {
            new_board.set_square::<true>(mov.to(), Piece::Queen, position.side_to_move);
        }
        MoveFlag::RookPromotion => {
            new_board.set_square::<false>(mov.to(), Piece::Rook, position.side_to_move);
        }
        MoveFlag::RookPromotionCapture => {
            new_board.set_square::<true>(mov.to(), Piece::Rook, position.side_to_move);
        }
        MoveFlag::BishopPromotion => {
            new_board.set_square::<false>(mov.to(), Piece::Bishop, position.side_to_move);
        }
        MoveFlag::BishopPromotionCapture => {
            new_board.set_square::<true>(mov.to(), Piece::Bishop, position.side_to_move);
        }
        MoveFlag::KnightPromotion => {
            new_board.set_square::<false>(mov.to(), Piece::Knight, position.side_to_move);
        }
        MoveFlag::KnightPromotionCapture => {
            new_board.set_square::<true>(mov.to(), Piece::Knight, position.side_to_move);
        }
        MoveFlag::DoublePawnPush => {
            new_board.set_square::<false>(mov.to(), piece, position.side_to_move);
            if UPDATE_METADATA {
                new_en_passant_square = Some(match position.side_to_move {
                    Color::White => {
                        Square::from((mov.to() as i8 + MoveDirection::SOUTH) as u8).unwrap()
                    }
                    Color::Black => {
                        Square::from((mov.to() as i8 + MoveDirection::NORTH) as u8).unwrap()
                    }
                });
            }
        }
        MoveFlag::Capture => {
            new_board.set_square::<true>(mov.to(), piece, position.side_to_move);
        }
        MoveFlag::Quiet => {
            new_board.set_square::<false>(mov.to(), piece, position.side_to_move);
        }
    }

    // Update castling rights
    if UPDATE_METADATA
        && !position.castling_rights.is_empty()
        && matches!(mov.flag(), MoveFlag::Capture | MoveFlag::Quiet)
    {
        match (position.side_to_move, piece, mov.from()) {
            (Color::White, Piece::King, Square::E1) => {
                new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
            }
            (Color::White, Piece::Rook, Square::A1) => {
                new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
            }
            (Color::White, Piece::Rook, Square::H1) => {
                new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
            }
            (Color::Black, Piece::King, Square::E8) => {
                new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
            }
            (Color::Black, Piece::Rook, Square::A8) => {
                new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
            }
            (Color::Black, Piece::Rook, Square::H8) => {
                new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
            }
            _ => (),
        }
    }

    Position {
        board: new_board,
        side_to_move: position.side_to_move.opposite(),
        en_passant_square: new_en_passant_square,
        castling_rights: new_castling_rights,
        halfmove_clock: if UPDATE_METADATA && (piece == Piece::Pawn || mov.flag().is_capture()) {
            0
        } else {
            position.halfmove_clock + 1
        },
        fullmove_number: if UPDATE_METADATA && position.side_to_move == Color::Black {
            position.fullmove_number + 1
        } else {
            position.fullmove_number
        },
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
        let new_position = super::make_move::<true>(
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
        let new_position = super::make_move::<true>(
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
        let new_position = super::make_move::<true>(
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

        let new_position = super::make_move::<true>(
            &position,
            Move::new(Square::C7, Square::C5, MoveFlag::DoublePawnPush),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq c6 0 2"
        );

        let new_position = super::make_move::<true>(
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
            position = super::make_move::<true>(&position, *mov);
        }

        assert_eq!(
            position.to_fen(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBP1P/2KR3q w kq - 0 3"
        );
    }
}
