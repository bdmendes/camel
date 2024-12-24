use crate::core::{piece::Piece, square::Square};

use gen::{piece_attacks, square_attackers};
use primitive_enum::primitive_enum;
use std::fmt::Display;

use super::{color::Color, Position};

pub mod gen;
pub mod make;
pub mod perft;

primitive_enum! { MoveFlag u8;
    Quiet,
    DoublePawnPush,
    KingsideCastle,
    QueensideCastle,
    Capture,
    EnpassantCapture,
    KnightPromotion = 8,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Move(pub u16);

impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((flag as u16) << 12))
    }

    pub fn is_reversible(&self) -> bool {
        self.flag() == MoveFlag::Quiet
    }

    pub fn from(&self) -> Square {
        Square::from((self.0 & 0x3F) as u8).unwrap()
    }

    pub fn to(&self) -> Square {
        Square::from(((self.0 & 0xFC0) >> 6) as u8).unwrap()
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from(((self.0 & 0xF000) >> 12) as u8).unwrap()
    }

    pub fn is_capture(&self) -> bool {
        ((1 << 14) & self.0) != 0
    }

    pub fn is_castle(&self) -> bool {
        self.0 == 2 || self.0 == 3
    }

    pub fn is_quiet(&self) -> bool {
        !self.is_capture() && self.promotion_piece().is_none()
    }

    pub fn promotion_piece(&self) -> Option<Piece> {
        if ((1 << 15) & self.0) == 0 {
            None
        } else {
            Some(match self.flag() {
                MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Piece::Knight,
                MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Piece::Bishop,
                MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Piece::Rook,
                _ => Piece::Queen,
            })
        }
    }

    pub fn is_pseudo_legal(&self, position: &Position) -> bool {
        let from_piece_color = position.piece_color_at(self.from());
        let to_color = position.color_at(self.to());
        let to_piece = position.piece_at(self.to());

        if let Some((piece, color)) = from_piece_color {
            // Basic legality assumptions. This is a good and fast start,
            // but not sufficient.
            if color != position.side_to_move
                || (to_color == Some(position.side_to_move) && !self.is_castle())
                || to_piece == Some(Piece::King)
                || (self.is_capture()
                    && self.flag() != MoveFlag::EnpassantCapture
                    && to_piece.is_none())
            {
                return false;
            }

            // Basic check test for king moves.
            if piece == Piece::King
                && !square_attackers(position, self.to(), position.side_to_move.flipped())
                    .is_empty()
            {
                return false;
            }

            match self.flag() {
                MoveFlag::Quiet if piece == Piece::Pawn => {
                    to_color.is_none() && self.to().file() == self.from().file()
                }
                MoveFlag::Quiet | MoveFlag::Capture => {
                    let attacks =
                        piece_attacks(piece, self.from(), position, position.side_to_move);
                    attacks.is_set(self.to())
                }
                MoveFlag::KingsideCastle => {
                    piece == Piece::King
                        && position.castling_rights().has_side(
                            position.side_to_move,
                            super::castling_rights::CastlingSide::Kingside,
                        )
                        && (!position.is_chess_960()).then(|| to_piece.is_none()).unwrap_or(true)
                }
                MoveFlag::QueensideCastle => {
                    piece == Piece::King
                        && position.castling_rights().has_side(
                            position.side_to_move,
                            super::castling_rights::CastlingSide::Queenside,
                        )
                        && (!position.is_chess_960()).then(|| to_piece.is_none()).unwrap_or(true)
                }
                MoveFlag::BishopPromotion
                | MoveFlag::KnightPromotion
                | MoveFlag::RookPromotion
                | MoveFlag::QueenPromotion => piece == Piece::Pawn && to_color.is_none(),
                MoveFlag::BishopPromotionCapture
                | MoveFlag::KnightPromotionCapture
                | MoveFlag::RookPromotionCapture
                | MoveFlag::QueenPromotionCapture => {
                    piece == Piece::Pawn && to_color == Some(position.side_to_move.flipped())
                }
                MoveFlag::EnpassantCapture => {
                    piece == Piece::Pawn
                        && to_color.is_none()
                        && position.ep_square() == Some(self.to())
                }
                MoveFlag::DoublePawnPush => {
                    piece == Piece::Pawn
                        && to_color.is_none()
                        && position
                            .color_at(self.to().shifted(match position.side_to_move {
                                Color::White => Square::SOUTH,
                                Color::Black => Square::NORTH,
                            }))
                            .is_none()
                }
            }
        } else {
            false
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from(),
            self.to(),
            self.promotion_piece()
                .map_or(String::new(), |p| format!("={}", p.to_string().to_uppercase()))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveFlag};
    use crate::core::moves::MoveFlag::*;
    use crate::core::piece::Piece::*;
    use crate::core::square::Square::*;
    use crate::core::{piece::Piece, square::Square};

    #[test]
    fn pack_unpack() {
        fn reflect(
            from: Square,
            to: Square,
            flag: MoveFlag,
            quiet: bool,
            capture: bool,
            promotion_piece: Option<Piece>,
        ) {
            let mov = Move::new(from, to, flag);
            assert_eq!(mov.from(), from);
            assert_eq!(mov.to(), to);
            assert_eq!(mov.flag(), flag);
            assert_eq!(mov.is_quiet(), quiet);
            assert_eq!(mov.is_capture(), capture);
            assert_eq!(mov.promotion_piece(), promotion_piece);
        }

        reflect(E4, H8, Quiet, true, false, None);
        reflect(E2, E4, DoublePawnPush, true, false, None);
        reflect(E1, G1, KingsideCastle, true, false, None);
        reflect(E8, C8, QueensideCastle, true, false, None);
        reflect(E4, E5, Capture, false, true, None);
        reflect(D5, C6, EnpassantCapture, false, true, None);
        reflect(E7, E8, KnightPromotion, false, false, Some(Knight));
        reflect(E7, E8, BishopPromotion, false, false, Some(Bishop));
        reflect(E7, E8, RookPromotion, false, false, Some(Rook));
        reflect(E7, E8, QueenPromotion, false, false, Some(Queen));
        reflect(E7, E8, KnightPromotionCapture, false, true, Some(Knight));
        reflect(E7, E8, BishopPromotionCapture, false, true, Some(Bishop));
        reflect(E7, E8, RookPromotionCapture, false, true, Some(Rook));
        reflect(E7, E8, QueenPromotionCapture, false, true, Some(Queen));
    }

    #[test]
    fn display() {
        let mov1 = Move::new(E4, E5, Quiet);
        assert_eq!(mov1.to_string(), "e4e5".to_string());

        let mov1 = Move::new(E7, D8, QueenPromotionCapture);
        assert_eq!(mov1.to_string(), "e7d8=Q".to_string());
    }
}
