use derive_more::{BitAnd, BitOr, Deref, DerefMut, Not};

use super::square::Square;

#[derive(Default, Debug, Hash, PartialEq, BitOr, BitAnd, Not, Deref, DerefMut, Copy, Clone)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn new(bb: u64) -> Self {
        Bitboard(bb)
    }

    pub fn pop_lsb(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        let lsb = self.0.trailing_zeros();
        self.0 &= self.0 - 1;
        Some(Square::try_from(lsb as u8).unwrap())
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << (square as u8);
    }

    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1 << (square as u8));
    }

    pub fn is_set(&self, square: Square) -> bool {
        self.0 & (1 << (square as u8)) != 0
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }
}
