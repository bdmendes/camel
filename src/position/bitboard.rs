use super::Square;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn is_set(&self, square: Square) -> bool {
        (self.0 & (1 << square as u64)) != 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square as u64;
    }

    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1 << square as u64);
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
        Square::from(lsb as u64)
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{bitboard::Bitboard, square::Square};

    #[test]
    fn set_unset() {
        let mut bb = Bitboard::default();
        assert!(!bb.is_set(Square::E4));

        bb.set(Square::E4);
        assert!(bb.is_set(Square::E4));

        bb.clear(Square::E4);
        assert!(!bb.is_set(Square::E4));
    }

    #[test]
    fn iter() {
        let bb = Bitboard(
            (1 << Square::E4 as u64) | (1 << Square::A6 as u64) | (1 << Square::H8 as u64),
        );

        let mut iter = bb.into_iter();
        assert_eq!(iter.next(), Some(Square::E4));
        assert_eq!(iter.next(), Some(Square::A6));
        assert_eq!(iter.next(), Some(Square::H8));
        assert_eq!(iter.next(), None);
    }
}
