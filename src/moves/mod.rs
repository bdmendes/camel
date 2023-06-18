use num_enum::TryFromPrimitive;

use crate::position::square::Square;

pub mod attacks;
pub mod gen;

#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MoveFlag {
    // Regular
    Quiet = 0b0000,
    Capture = 0b0001,

    // King specials
    KingCastle = 0b0010,
    QueenCastle = 0b0011,

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
}

impl MoveFlag {
    pub fn is_quiet(&self) -> bool {
        match self {
            Self::Quiet | Self::DoublePawnPush | Self::KingCastle | Self::QueenCastle => true,
            _ => false,
        }
    }

    pub fn is_capture(&self) -> bool {
        match self {
            Self::Capture
            | Self::EnPassantCapture
            | Self::KnightPromotionCapture
            | Self::BishopPromotionCapture
            | Self::RookPromotionCapture
            | Self::QueenPromotionCapture => true,
            _ => false,
        }
    }

    pub fn is_promotion(&self) -> bool {
        match self {
            Self::KnightPromotion
            | Self::BishopPromotion
            | Self::RookPromotion
            | Self::QueenPromotion
            | Self::KnightPromotionCapture
            | Self::BishopPromotionCapture
            | Self::RookPromotionCapture
            | Self::QueenPromotionCapture => true,
            _ => false,
        }
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
        Square::try_from((self.data & 0b111111) as u8).unwrap()
    }

    pub fn to(&self) -> Square {
        Square::try_from(((self.data >> 6) & 0b111111) as u8).unwrap()
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::try_from((self.data >> 12) as u8).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::square::Square;

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
}
