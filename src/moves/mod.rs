use self::{
    attacks::specials::pawn_attacks,
    gen::{piece_attacks, MoveDirection},
};
use crate::position::{
    bitboard::Bitboard, board::Piece, square::Square, CastlingRights, Color, Position,
};

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

    pub fn is_castle(&self) -> bool {
        matches!(self, Self::KingsideCastle | Self::QueensideCastle)
    }

    pub fn promotion_piece(&self) -> Option<Piece> {
        match self {
            Self::QueenPromotion | Self::QueenPromotionCapture => Some(Piece::Queen),
            Self::RookPromotion | Self::RookPromotionCapture => Some(Piece::Rook),
            Self::KnightPromotion | Self::KnightPromotionCapture => Some(Piece::Knight),
            Self::BishopPromotion | Self::BishopPromotionCapture => Some(Piece::Bishop),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Move(u16);

impl Move {
    pub fn new_raw(bytes: u16) -> Self {
        Move(bytes)
    }

    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((flag as u16) << 12))
    }

    pub fn from(&self) -> Square {
        Square::from((self.0 & 0b111111) as u8).unwrap()
    }

    pub fn to(&self) -> Square {
        Square::from(((self.0 >> 6) & 0b111111) as u8).unwrap()
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from((self.0 >> 12) as u8).unwrap()
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

    pub fn raw(&self) -> u16 {
        self.0
    }

    pub fn is_pseudo_legal(&self, position: &Position) -> bool {
        let from_piece_color = position.board.piece_color_at(self.from());
        let to_color = position.board.color_at(self.to());
        let to_piece = position.board.piece_at(self.to());

        if let Some((piece, color)) = from_piece_color {
            // Basic legality assumptions. This is a good and fast start,
            // but not sufficient.
            if color != position.side_to_move
                || (to_color == Some(position.side_to_move) && !self.flag().is_castle())
                || to_piece == Some(Piece::King)
                || (self.flag().is_capture()
                    && self.flag() != MoveFlag::EnPassantCapture
                    && to_piece.is_none())
            {
                return false;
            }

            match self.flag() {
                MoveFlag::Quiet if piece == Piece::Pawn => {
                    to_color.is_none() && self.to().file() == self.from().file()
                }
                MoveFlag::Quiet | MoveFlag::Capture => {
                    let attacks = piece_attacks(
                        piece,
                        self.from(),
                        position.board.occupancy_bb_all(),
                        position.side_to_move,
                    );
                    attacks.is_set(self.to())
                }
                MoveFlag::KingsideCastle => {
                    let castle_right = match position.side_to_move {
                        Color::White => CastlingRights::WHITE_KINGSIDE,
                        Color::Black => CastlingRights::BLACK_KINGSIDE,
                    };
                    piece == Piece::King
                        && position.castling_rights.contains(castle_right)
                        && (!position.is_chess960).then(|| to_piece.is_none()).unwrap_or(true)
                }
                MoveFlag::QueensideCastle => {
                    let castle_right = match position.side_to_move {
                        Color::White => CastlingRights::WHITE_QUEENSIDE,
                        Color::Black => CastlingRights::BLACK_QUEENSIDE,
                    };
                    piece == Piece::King
                        && position.castling_rights.contains(castle_right)
                        && (!position.is_chess960).then(|| to_piece.is_none()).unwrap_or(true)
                }
                MoveFlag::BishopPromotion
                | MoveFlag::KnightPromotion
                | MoveFlag::RookPromotion
                | MoveFlag::QueenPromotion => piece == Piece::Pawn && to_color.is_none(),
                MoveFlag::BishopPromotionCapture
                | MoveFlag::KnightPromotionCapture
                | MoveFlag::RookPromotionCapture
                | MoveFlag::QueenPromotionCapture => {
                    piece == Piece::Pawn && to_color == Some(position.side_to_move.opposite())
                }
                MoveFlag::EnPassantCapture => {
                    piece == Piece::Pawn
                        && to_color.is_none()
                        && position.en_passant_square == Some(self.to())
                }
                MoveFlag::DoublePawnPush => {
                    piece == Piece::Pawn
                        && to_color.is_none()
                        && self
                            .to()
                            .shift(match position.side_to_move {
                                Color::White => MoveDirection::SOUTH,
                                Color::Black => MoveDirection::NORTH,
                            })
                            .map_or(false, |sq| position.board.color_at(sq).is_none())
                }
            }
        } else {
            false
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

pub fn make_move(position: &Position, mov: Move) -> Position {
    let mut new_board = position.board;
    let mut new_castling_rights = position.castling_rights;
    let mut new_en_passant_square = None;

    let piece = new_board.piece_at(mov.from()).unwrap();
    let mov_flag = mov.flag();

    new_board.clear_square(mov.from());

    if mov_flag.is_castle() {
        match mov_flag {
            MoveFlag::KingsideCastle => match position.side_to_move {
                Color::White => {
                    let right_hand_side_rook =
                        position.is_chess960.then(|| mov.to()).unwrap_or(Square::H1);
                    new_board.clear_square(right_hand_side_rook);
                    new_board.set_square(Square::G1, Piece::King, Color::White);
                    new_board.set_square(Square::F1, Piece::Rook, Color::White);
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                Color::Black => {
                    let right_hand_side_rook =
                        position.is_chess960.then(|| mov.to()).unwrap_or(Square::H8);
                    new_board.clear_square(right_hand_side_rook);
                    new_board.set_square(Square::G8, Piece::King, Color::Black);
                    new_board.set_square(Square::F8, Piece::Rook, Color::Black);
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
            },
            MoveFlag::QueensideCastle => match position.side_to_move {
                Color::White => {
                    let left_hand_side_rook =
                        position.is_chess960.then(|| mov.to()).unwrap_or(Square::A1);
                    new_board.clear_square(left_hand_side_rook);
                    new_board.set_square(Square::C1, Piece::King, Color::White);
                    new_board.set_square(Square::D1, Piece::Rook, Color::White);
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                Color::Black => {
                    let left_hand_side_rook =
                        position.is_chess960.then(|| mov.to()).unwrap_or(Square::A8);
                    new_board.clear_square(left_hand_side_rook);
                    new_board.set_square(Square::C8, Piece::King, Color::Black);
                    new_board.set_square(Square::D8, Piece::Rook, Color::Black);
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
            },
            _ => unreachable!(),
        }
    } else if let Some(promotion_piece) = mov_flag.promotion_piece() {
        new_board.set_square(mov.to(), promotion_piece, position.side_to_move);
    } else if mov_flag == MoveFlag::DoublePawnPush {
        new_board.set_square(mov.to(), piece, position.side_to_move);

        let candidate_en_passant = match position.side_to_move {
            Color::White => mov.to().shift(MoveDirection::SOUTH).unwrap(),
            Color::Black => mov.to().shift(MoveDirection::NORTH).unwrap(),
        };
        if pawn_attacks(&position.board, position.side_to_move.opposite())
            .is_set(candidate_en_passant)
        {
            new_en_passant_square = Some(candidate_en_passant);
        }
    } else if mov_flag == MoveFlag::EnPassantCapture {
        new_board.set_square(mov.to(), Piece::Pawn, position.side_to_move);
        new_board.clear_square(match position.side_to_move {
            Color::White => mov.to().shift(MoveDirection::SOUTH).unwrap(),
            Color::Black => mov.to().shift(MoveDirection::NORTH).unwrap(),
        });
    } else {
        new_board.set_square(mov.to(), piece, position.side_to_move);

        if !position.castling_rights.is_empty() && matches!(piece, Piece::Rook | Piece::King) {
            let king_square = match position.side_to_move {
                Color::White => (position.board.pieces_bb_color(Piece::King, Color::White)).next(),
                Color::Black => (position.board.pieces_bb_color(Piece::King, Color::Black)).next(),
            };
            let king_rank_rooks = match position.side_to_move {
                Color::White => {
                    Bitboard::rank_mask(0)
                        & position.board.pieces_bb_color(Piece::Rook, Color::White)
                }
                Color::Black => {
                    Bitboard::rank_mask(7)
                        & position.board.pieces_bb_color(Piece::Rook, Color::Black)
                }
            };
            let left_hand_side_rook = king_rank_rooks
                .into_iter()
                .next()
                .filter(|sq| sq.file() < king_square.map_or(0, Square::file));
            let right_hand_side_rook = king_rank_rooks
                .into_iter()
                .next_back()
                .filter(|sq| sq.file() > king_square.map_or(7, Square::file));

            match (position.side_to_move, piece, mov.from()) {
                (Color::White, Piece::King, _) => {
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                (Color::Black, Piece::King, _) => {
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
                (Color::White, Piece::Rook, square) if Some(square) == left_hand_side_rook => {
                    new_castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                }
                (Color::White, Piece::Rook, square) if Some(square) == right_hand_side_rook => {
                    new_castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                }
                (Color::Black, Piece::Rook, square) if Some(square) == left_hand_side_rook => {
                    new_castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                }
                (Color::Black, Piece::Rook, square) if Some(square) == right_hand_side_rook => {
                    new_castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                }
                _ => (),
            }
        }
    }

    Position {
        board: new_board,
        side_to_move: position.side_to_move.opposite(),
        en_passant_square: new_en_passant_square,
        castling_rights: new_castling_rights,
        halfmove_clock: if piece == Piece::Pawn || mov_flag.is_capture() {
            0
        } else {
            position.halfmove_clock.saturating_add(1)
        },
        fullmove_number: if position.side_to_move == Color::Black {
            position.fullmove_number.saturating_add(1)
        } else {
            position.fullmove_number
        },
        is_chess960: position.is_chess960,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{
        fen::{FromFen, ToFen, KIWIPETE_BLACK_FEN, KIWIPETE_WHITE_FEN, START_FEN},
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
        let new_position =
            super::make_move(&position, Move::new(Square::D5, Square::E6, MoveFlag::Capture));
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p1ppqpb1/bn2Pnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1"
        );
    }

    #[test]
    fn make_move_castle_long() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let new_position = super::make_move(
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
        let new_position = super::make_move(
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

        let new_position = super::make_move(
            &position,
            Move::new(Square::C7, Square::C5, MoveFlag::DoublePawnPush),
        );
        assert_eq!(
            new_position.to_fen(),
            "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq c6 0 2"
        );

        let new_position = super::make_move(
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
            position = super::make_move(&position, *mov);
        }

        assert_eq!(
            position.to_fen(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBP1P/2KR3q w kq - 0 3"
        );
    }

    #[test]
    fn pseudo_legal_chess() {
        let start_position = Position::from_fen(START_FEN).unwrap();
        let start_moves = start_position.moves(gen::MoveStage::All);

        let kiwipete_position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let kiwipete_moves = kiwipete_position.moves(gen::MoveStage::All);

        for mov in &start_moves {
            assert!(mov.is_pseudo_legal(&start_position));
            if !kiwipete_moves.contains(mov) {
                assert!(!mov.is_pseudo_legal(&kiwipete_position));
            }
        }

        for mov in &kiwipete_moves {
            assert!(mov.is_pseudo_legal(&kiwipete_position));
            if !start_moves.contains(mov) {
                assert!(!mov.is_pseudo_legal(&start_position));
            }
        }
    }

    #[test]
    fn pseudo_legal_chess960() {
        let position = Position::from_fen(
            "q1krnrb1/ppp2pbp/2n1p1p1/3p4/3P1PP1/2NNP3/PPP4P/Q1RK1RBB w KQ - 0 7",
        )
        .unwrap();
        let moves = position.moves(gen::MoveStage::All);

        for mov in &moves {
            assert!(mov.is_pseudo_legal(&position));
        }
    }
}
