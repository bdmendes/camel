use std::{
    array,
    fmt::{Display, Write},
};

use ctor::ctor;
use derive_more::derive::{BitAnd, BitOr, Not, Shl, ShlAssign, Shr, ShrAssign};

use super::{square::Direction, Square};

#[ctor]
static FILE_MASK: [Bitboard; 8] = {
    array::from_fn(|idx| {
        let mut bb = Bitboard(0);
        for rank in 0..=7 {
            bb.set(Square::from((idx + rank * 8) as u8).unwrap());
        }
        bb
    })
};

#[ctor]
static RANK_MASK: [Bitboard; 8] = {
    array::from_fn(|idx| {
        let mut bb = Bitboard(0);
        for file in 0..=7 {
            bb.set(Square::from((file + idx * 8) as u8).unwrap());
        }
        bb
    })
};

#[derive(
    Default, Copy, Clone, Debug, PartialEq, Eq, BitOr, BitAnd, Shl, Shr, ShlAssign, ShrAssign, Not,
)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn empty() -> Self {
        Bitboard(0)
    }

    pub fn from_square(square: Square) -> Self {
        Bitboard(1 << square as u64)
    }

    pub fn is_set(&self, square: Square) -> bool {
        (self.0 & (1 << square as u64)) != 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square as u64;
    }

    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1 << square as u64);
    }

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn shift(&self, direction: Direction) -> Self {
        if direction >= 0 {
            Bitboard(self.0 << direction)
        } else {
            Bitboard(self.0 >> (-direction))
        }
    }

    pub fn file_mask(file: u8) -> Self {
        FILE_MASK[file.min(7) as usize]
    }

    pub fn rank_mask(rank: u8) -> Self {
        RANK_MASK[rank.min(7) as usize]
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
}

impl DoubleEndedIterator for Bitboard {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let msb = 63 - self.0.leading_zeros();
        self.0 &= !(1 << msb);
        Square::from(msb as u8)
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                f.write_char(
                    if self.is_set(Square::from_file_rank(file, rank).unwrap()) {
                        '1'
                    } else {
                        '0'
                    },
                )?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{bitboard::Bitboard, square::Square};

    #[test]
    fn from_square() {
        let square: Bitboard = Bitboard::from_square(Square::C1);
        assert_eq!(square.0, 1 << 2);
    }

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
        let bb = Bitboard::from_square(Square::E4)
            | Bitboard::from_square(Square::A6)
            | Bitboard::from_square(Square::H8);

        let mut iter = bb.into_iter();
        assert_eq!(iter.next(), Some(Square::E4));
        assert_eq!(iter.next(), Some(Square::A6));
        assert_eq!(iter.next(), Some(Square::H8));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_rev() {
        let bb = Bitboard::from_square(Square::E4)
            | Bitboard::from_square(Square::A6)
            | Bitboard::from_square(Square::H8);

        let mut iter = bb.into_iter().rev();
        assert_eq!(iter.next(), Some(Square::H8));
        assert_eq!(iter.next(), Some(Square::A6));
        assert_eq!(iter.next(), Some(Square::E4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn shift() {
        let bb = Bitboard::from_square(Square::E4) | Bitboard::from_square(Square::D4);

        assert_eq!(
            bb.shift(Square::NORTH),
            Bitboard::from_square(Square::E5) | Bitboard::from_square(Square::D5)
        );

        assert_eq!(
            bb.shift(2 * Square::SOUTH + Square::WEST),
            Bitboard::from_square(Square::D2) | Bitboard::from_square(Square::C2)
        );
    }

    #[test]
    fn masks() {
        assert_eq!(
            Bitboard::file_mask(2),
            Bitboard::from_square(Square::C1)
                | Bitboard::from_square(Square::C2)
                | Bitboard::from_square(Square::C3)
                | Bitboard::from_square(Square::C4)
                | Bitboard::from_square(Square::C5)
                | Bitboard::from_square(Square::C6)
                | Bitboard::from_square(Square::C7)
                | Bitboard::from_square(Square::C8)
        );
        assert_eq!(Bitboard::file_mask(30), Bitboard::file_mask(7));

        assert_eq!(
            Bitboard::rank_mask(2),
            Bitboard::from_square(Square::A3)
                | Bitboard::from_square(Square::B3)
                | Bitboard::from_square(Square::C3)
                | Bitboard::from_square(Square::D3)
                | Bitboard::from_square(Square::E3)
                | Bitboard::from_square(Square::F3)
                | Bitboard::from_square(Square::G3)
                | Bitboard::from_square(Square::H3)
        );
        assert_eq!(Bitboard::rank_mask(30), Bitboard::rank_mask(7));
    }

    #[test]
    fn display() {
        let bb = Bitboard::from_square(Square::E4)
            | Bitboard::from_square(Square::A6)
            | Bitboard::from_square(Square::H8);
        let str = bb.to_string();
        let lines = str.lines().collect::<Vec<&str>>();
        assert_eq!(lines[0], "00000001");
        assert_eq!(lines[1], "00000000");
        assert_eq!(lines[2], "10000000");
        assert_eq!(lines[3], "00000000");
        assert_eq!(lines[4], "00001000");
        assert_eq!(lines[5], "00000000");
        assert_eq!(lines[6], "00000000");
        assert_eq!(lines[7], "00000000");
    }
}
