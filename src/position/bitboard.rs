use super::square::Square;
use derive_more::{BitAnd, BitOr, Deref, DerefMut, Not};
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Hash, PartialEq, BitOr, BitAnd, Not, Deref, DerefMut, Copy, Clone, Eq)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn new(bb: u64) -> Self {
        Bitboard(bb)
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

    pub const fn shift(&self, shift_value: i8) -> Self {
        if shift_value >= 0 {
            Bitboard(self.0 << shift_value)
        } else {
            Bitboard(self.0 >> -shift_value)
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_not_empty(&self) -> bool {
        self.0 != 0
    }

    pub const fn file_mask(file: u8) -> Self {
        debug_assert!(file < 8);
        Bitboard(0x0101010101010101 << file)
    }

    pub const fn rank_mask(rank: u8) -> Self {
        debug_assert!(rank < 8);
        Bitboard(0xFF << (rank * 8))
    }

    pub fn files_mask_left(file: u8) -> Self {
        debug_assert!(file < 8);
        (0..file).fold(Bitboard::new(0), |acc, file| acc | Bitboard::file_mask(file))
    }

    pub fn files_mask_right(file: u8) -> Self {
        debug_assert!(file < 8);
        (file + 1..8).fold(Bitboard::new(0), |acc, file| acc | Bitboard::file_mask(file))
    }

    pub fn ranks_mask_up(rank: u8) -> Self {
        debug_assert!(rank < 8);
        (rank + 1..8).fold(Bitboard::new(0), |acc, rank| acc | Bitboard::rank_mask(rank))
    }

    pub fn ranks_mask_down(rank: u8) -> Self {
        debug_assert!(rank < 8);
        (0..rank).fold(Bitboard::new(0), |acc, rank| acc | Bitboard::rank_mask(rank))
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let lsb = self.0.trailing_zeros();
        self.0 &= self.0 - 1;
        Square::from(lsb as u8)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.0.count_ones() as usize;
        (count, Some(count))
    }
}

impl DoubleEndedIterator for Bitboard {
    fn next_back(&mut self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        let msb = 63 - self.0.leading_zeros();
        self.0 &= !(1 << msb);
        Square::from(msb as u8)
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bb = self.0;
        for rank in (0..8).rev() {
            for file in 0..8 {
                if bb & (1 << (rank * 8 + file)) != 0 {
                    write!(f, "1")?;
                } else {
                    write!(f, "0")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::position::square::Square;

    use super::Bitboard;

    #[test]
    fn pop_lsb() {
        let mut bb = Bitboard::new(0b0000_1011_0011).into_iter();

        assert_eq!(bb.next(), Square::from(0));
        assert_eq!(bb.next(), Square::from(1));
        assert_eq!(bb.next(), Square::from(4));
        assert_eq!(bb.next(), Square::from(5));
        assert_eq!(bb.next(), Square::from(7));
        assert_eq!(bb.next(), None);
    }

    #[test]
    fn pop_msb() {
        let mut bb = Bitboard::new(0b0000_1011_0011).into_iter().rev();

        assert_eq!(bb.next(), Square::from(7));
        assert_eq!(bb.next(), Square::from(5));
        assert_eq!(bb.next(), Square::from(4));
        assert_eq!(bb.next(), Square::from(1));
        assert_eq!(bb.next(), Square::from(0));
        assert_eq!(bb.next(), None);
    }

    #[test]
    fn single_masks() {
        let square = Square::E4;
        assert_eq!(Bitboard::file_mask(square.file()), Bitboard::new(0x10_10_10_10_10_10_10_10));
        assert_eq!(Bitboard::rank_mask(square.rank()), Bitboard::new(0x00_00_00_00_FF_00_00_00));
    }

    #[test]
    fn files_masks() {
        let square = Square::E4;
        assert_eq!(
            Bitboard::files_mask_left(square.file(),),
            Bitboard::new(0x0F_0F_0F_0F_0F_0F_0F_0F)
        );
        assert_eq!(
            Bitboard::files_mask_right(square.file(),),
            Bitboard::new(0xE0_E0_E0_E0_E0_E0_E0_E0)
        );
    }

    #[test]
    fn ranks_masks() {
        let square = Square::E4;
        assert_eq!(
            Bitboard::ranks_mask_down(square.rank(),),
            Bitboard::new(0x00_00_00_00_00_FF_FF_FF)
        );
        assert_eq!(
            Bitboard::ranks_mask_up(square.rank(),),
            Bitboard::new(0xFF_FF_FF_FF_00_00_00_00)
        );
    }
}
