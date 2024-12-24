use primitive_enum::primitive_enum;
use std::{
    fmt::Display,
    ops::{Shl, Shr},
    str::FromStr,
};

use super::Color;

pub type Direction = i8;

const SQUARE_COLORS: [Color; 64] = {
    const WHITE_SQUARES: u64 = 0x55_AA_55_AA_55_AA_55_AA;
    let mut arr = [Color::White; 64];
    let mut sq = 0;
    while sq < 64 {
        if ((1 << sq) & WHITE_SQUARES) == 0 {
            arr[sq] = Color::Black
        }
        sq += 1;
    }
    arr
};

#[rustfmt::skip]
primitive_enum! { Square u8;
    A1=0, B1=1, C1, D1, E1, F1, G1, H1,
    A2=8, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub const NORTH: Direction = 8;
    pub const SOUTH: Direction = -8;
    pub const WEST: Direction = -1;
    pub const EAST: Direction = 1;

    pub fn flip(self) -> Square {
        let file = self.file();
        let rank = 7 - self.rank();
        Square::from(rank * 8 + file).unwrap()
    }

    pub fn from_file_rank(file: u8, rank: u8) -> Option<Self> {
        if file >= 8 || rank >= 8 {
            None
        } else {
            Square::from(rank * 8 + file)
        }
    }

    pub const fn pawn_direction(color: Color) -> i8 {
        match color {
            Color::White => Self::NORTH,
            Color::Black => Self::SOUTH,
        }
    }

    pub const fn color(self) -> Color {
        SQUARE_COLORS[self as usize]
    }

    pub fn manhattan_distance(self, other: Square) -> u8 {
        let file_diff = (self.file() as i8 - other.file() as i8).unsigned_abs();
        let rank_diff = (self.rank() as i8 - other.rank() as i8).unsigned_abs();
        file_diff + rank_diff
    }

    pub const fn rank(self) -> u8 {
        (self as u8) / 8
    }

    pub const fn file(self) -> u8 {
        (self as u8) % 8
    }

    pub fn shifted(self, direction: Direction) -> Self {
        if direction >= 0 {
            self << direction as u8
        } else {
            self >> (-direction) as u8
        }
    }
}

impl Shr<u8> for Square {
    type Output = Square;

    fn shr(self, rhs: u8) -> Self::Output {
        Square::from((self as u8).saturating_sub(rhs)).unwrap()
    }
}

impl Shl<u8> for Square {
    type Output = Square;

    fn shl(self, lhs: u8) -> Self::Output {
        Square::from((self as u8).saturating_add(lhs).min(63)).unwrap()
    }
}

impl FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let file = chars.next().ok_or(())?;
        let file: u8 = match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err(()),
        };

        let rank = chars.next().ok_or(())?;
        let rank: u8 = rank.to_digit(10).ok_or(())? as u8;
        if !((1..=8).contains(&rank)) {
            return Err(());
        }

        Ok(Square::from((rank - 1) * 8 + file).unwrap())
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file = match self.file() {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            _ => 'h',
        };
        let rank = self.rank() + 1;
        write!(f, "{file}{rank}")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::core::{square::Square, Color};

    #[test]
    fn color() {
        assert_eq!(Square::A1.color(), Color::Black);
        assert_eq!(Square::A8.color(), Color::White);
        assert_eq!(Square::H1.color(), Color::White);
        assert_eq!(Square::H8.color(), Color::Black);
        assert_eq!(Square::E4.color(), Color::White);
        assert_eq!(Square::D5.color(), Color::White);
        assert_eq!(Square::D6.color(), Color::Black);
        assert_eq!(Square::D4.color(), Color::Black);
        assert_eq!(Square::E5.color(), Color::Black);
    }

    #[test]
    fn file() {
        assert_eq!(Square::A1.file(), 0);
        assert_eq!(Square::A8.file(), 0);
        assert_eq!(Square::H1.file(), 7);
        assert_eq!(Square::H8.file(), 7);
        assert_eq!(Square::E4.file(), 4);
        assert_eq!(Square::D5.file(), 3);
    }

    #[test]
    fn rank() {
        assert_eq!(Square::A1.rank(), 0);
        assert_eq!(Square::A8.rank(), 7);
        assert_eq!(Square::H1.rank(), 0);
        assert_eq!(Square::H8.rank(), 7);
        assert_eq!(Square::E4.rank(), 3);
        assert_eq!(Square::D5.rank(), 4);
    }

    #[test]
    fn shift() {
        assert_eq!(Square::E4 >> 8, Square::E3);
        assert_eq!(Square::E4 >> 64, Square::A1);
        assert_eq!(Square::E4 << 8, Square::E5);
        assert_eq!(Square::E4 << 64, Square::H8);

        assert_eq!(Square::E4.shifted(Square::NORTH), Square::E5);
        assert_eq!(Square::E4.shifted(Square::SOUTH), Square::E3);
    }

    #[test]
    fn from_str() {
        assert_eq!(Square::from_str("a1"), Ok(Square::A1));
        assert_eq!(Square::from_str("a8"), Ok(Square::A8));
        assert_eq!(Square::from_str("h1"), Ok(Square::H1));
        assert_eq!(Square::from_str("h8"), Ok(Square::H8));
        assert_eq!(Square::from_str("e4"), Ok(Square::E4));
        assert_eq!(Square::from_str("d5"), Ok(Square::D5));

        assert!(Square::from_str("").is_err());
        assert!(Square::from_str("a").is_err());
        assert!(Square::from_str("a9").is_err());
        assert!(Square::from_str("i1").is_err());
    }

    #[test]
    fn from_file_rank() {
        assert_eq!(Square::from_file_rank(4, 3), Some(Square::E4));
        assert_eq!(Square::from_file_rank(6, 3), Some(Square::G4));
        assert_eq!(Square::from_file_rank(3, 4), Some(Square::D5));
        assert_eq!(Square::from_file_rank(8, 0), None);
        assert_eq!(Square::from_file_rank(0, 8), None);
    }

    #[test]
    fn display() {
        assert_eq!(Square::A1.to_string(), "a1");
        assert_eq!(Square::A8.to_string(), "a8");
        assert_eq!(Square::H1.to_string(), "h1");
        assert_eq!(Square::H8.to_string(), "h8");
        assert_eq!(Square::E4.to_string(), "e4");
        assert_eq!(Square::D5.to_string(), "d5");
    }
}
