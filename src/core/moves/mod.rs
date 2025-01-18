use crate::core::{piece::Piece, square::Square};

use primitive_enum::primitive_enum;
use std::fmt::Display;

pub mod gen;
pub mod make;
pub mod perft;
pub mod see;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move(u16);

impl Move {
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((flag as u16) << 12))
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
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from(),
            self.to(),
            self.promotion_piece().map_or(String::new(), |p| p.to_string())
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
        assert_eq!(mov1.to_string(), "e7d8q".to_string());
    }
}
