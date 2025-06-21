use super::{Square, square::Direction};
use derive_more::derive::{BitAnd, BitOr, Not, Shl, ShlAssign, Shr, ShrAssign};
use std::fmt::{Display, Write};

const FILE_MASK: [Bitboard; 8] = {
    let mut arr = [Bitboard::empty(); 8];
    let mut file = 0;
    while file < 8 {
        arr[file] = Bitboard(0x0101010101010101 << file);
        file += 1;
    }
    arr
};

const RANK_MASK: [Bitboard; 8] = {
    let mut arr = [Bitboard::empty(); 8];
    let mut rank = 0;
    while rank < 8 {
        arr[rank] = Bitboard(0xFF << (rank * 8));
        rank += 1;
    }
    arr
};

static BETWEEN: [Bitboard; 64 * 64] = {
    let mut arr = [0u64; 64 * 64];

    let mut from = 0;
    while from < 64 {
        let from_rank = from / 8;
        let from_file = from % 8;

        let mut to = 0;
        while to < 64 {
            if from != to {
                let to_rank = to / 8;
                let to_file = to % 8;
                let idx = from * 64 + to;
                let mut bb = 0u64;

                // Same rank
                if from_rank == to_rank {
                    let (start, end) = if from_file < to_file {
                        (from_file + 1, to_file)
                    } else {
                        (to_file + 1, from_file)
                    };
                    let mut file = start;
                    while file < end {
                        bb |= 1 << (from_rank * 8 + file);
                        file += 1;
                    }
                }
                // Same file
                else if from_file == to_file {
                    let (start, end) = if from_rank < to_rank {
                        (from_rank + 1, to_rank)
                    } else {
                        (to_rank + 1, from_rank)
                    };
                    let mut rank = start;
                    while rank < end {
                        bb |= 1 << (rank * 8 + from_file);
                        rank += 1;
                    }
                }
                // Same diagonal (or anti-diagonal)
                else if (from_rank as i16 - to_rank as i16).abs()
                    == (from_file as i16 - to_file as i16).abs()
                {
                    let rank_step = if to_rank > from_rank { 1i16 } else { -1i16 };
                    let file_step = if to_file > from_file { 1i16 } else { -1i16 };
                    let mut r = from_rank as i16 + rank_step;
                    let mut f = from_file as i16 + file_step;

                    while r != to_rank as i16 && f != to_file as i16 {
                        bb |= 1 << (r as u8 * 8 + f as u8);
                        r += rank_step;
                        f += file_step;
                    }
                }

                arr[idx] = bb;
            }
            to += 1;
        }
        from += 1;
    }

    let mut ret_arr = [Bitboard::empty(); 64 * 64];
    let mut i = 0;
    while i < 64 * 64 {
        ret_arr[i] = Bitboard::new(arr[i]);
        i += 1;
    }
    ret_arr
};

#[derive(
    Default, Copy, Clone, Debug, PartialEq, Eq, BitOr, BitAnd, Shl, Shr, ShlAssign, ShrAssign, Not,
)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const fn empty() -> Self {
        Bitboard(0)
    }

    pub const fn full() -> Self {
        Bitboard(u64::MAX)
    }

    pub const fn new(data: u64) -> Self {
        Bitboard(data)
    }

    pub const fn raw(&self) -> u64 {
        self.0
    }

    pub fn lsb(&self) -> Option<Square> {
        let lsb = self.0.trailing_zeros();
        Square::from(lsb as u8)
    }

    pub fn msb(&self) -> Option<Square> {
        let msb = 63_u32.wrapping_sub(self.0.leading_zeros());
        Square::from(msb as u8)
    }

    pub const fn from_square(square: Square) -> Self {
        Bitboard(1 << square as usize)
    }

    pub const fn is_set(&self, square: Square) -> bool {
        (self.0 & 1 << square as usize) != 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= 1 << square as usize;
    }

    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1 << square as usize);
    }

    pub const fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub const fn shifted(&self, direction: Direction) -> Self {
        let shift = direction & 63;
        Bitboard((self.0 << shift) | (self.0 >> (-shift & 63)))
    }

    pub const fn file_mask(file: u8) -> Self {
        FILE_MASK[file as usize]
    }

    pub const fn rank_mask(rank: u8) -> Self {
        RANK_MASK[rank as usize]
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn between(from: Square, to: Square) -> Bitboard {
        BETWEEN[from as usize * 64 + to as usize]
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(lsb) = self.lsb() {
            self.0 &= self.0 - 1;
            Some(lsb)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for Bitboard {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(msb) = self.msb() {
            self.0 &= !(1 << msb as u8);
            Some(msb)
        } else {
            None
        }
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
    use crate::core::position::{bitboard::Bitboard, square::Square};

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
    fn lsb() {
        let bb = Bitboard::from_square(Square::E4)
            | Bitboard::from_square(Square::A6)
            | Bitboard::from_square(Square::H8);
        assert_eq!(bb.lsb(), Some(Square::E4));
        assert_eq!(Bitboard::empty().lsb(), None);
    }

    #[test]
    fn msb() {
        let bb = Bitboard::from_square(Square::E4)
            | Bitboard::from_square(Square::A6)
            | Bitboard::from_square(Square::H8);
        assert_eq!(bb.msb(), Some(Square::H8));
        assert_eq!(Bitboard::empty().msb(), None);
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
            bb.shifted(Square::NORTH),
            Bitboard::from_square(Square::E5) | Bitboard::from_square(Square::D5)
        );

        assert_eq!(
            bb.shifted(2 * Square::SOUTH + Square::WEST),
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
    }

    #[test]
    fn between_row() {
        assert_eq!(
            Bitboard::between(Square::E1, Square::H1),
            Bitboard::from_square(Square::F1) | Bitboard::from_square(Square::G1)
        );

        assert_eq!(
            Bitboard::between(Square::H1, Square::E1),
            Bitboard::from_square(Square::F1) | Bitboard::from_square(Square::G1)
        );

        assert_eq!(Bitboard::between(Square::E1, Square::E1), Bitboard::empty());
    }

    #[test]
    fn between_file() {
        assert_eq!(
            Bitboard::between(Square::E1, Square::E4),
            Bitboard::from_square(Square::E2) | Bitboard::from_square(Square::E3)
        );

        assert_eq!(
            Bitboard::between(Square::E4, Square::E1),
            Bitboard::from_square(Square::E2) | Bitboard::from_square(Square::E3)
        );
    }

    #[test]
    fn between_diagonal() {
        assert_eq!(
            Bitboard::between(Square::E1, Square::B4),
            Bitboard::from_square(Square::D2) | Bitboard::from_square(Square::C3)
        );

        assert_eq!(
            Bitboard::between(Square::B4, Square::E1),
            Bitboard::from_square(Square::D2) | Bitboard::from_square(Square::C3)
        );

        assert_eq!(
            Bitboard::between(Square::E1, Square::H4),
            Bitboard::from_square(Square::F2) | Bitboard::from_square(Square::G3)
        );

        assert_eq!(
            Bitboard::between(Square::H4, Square::E1),
            Bitboard::from_square(Square::F2) | Bitboard::from_square(Square::G3)
        );
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
