use num_enum::TryFromPrimitive;

use crate::position::square::Square;

#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MoveFlag {
    Quiet = 0b000,
    Capture = 0b001,
    QueenPromotion = 0b010,
    KnightPromotion = 0b011,
    BishopPromotion = 0b100,
    RookPromotion = 0b101,
}

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
