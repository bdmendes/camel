use primitive_enum::primitive_enum;

#[rustfmt::skip]
primitive_enum!(
    Square u8;
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
);

impl Square {
    pub const fn rank(self) -> u8 {
        (self as u8) / 8
    }

    pub const fn file(self) -> u8 {
        (self as u8) % 8
    }

    pub fn flip(self) -> Square {
        let file = self.file();
        let rank = 7 - self.rank();
        Square::from(rank * 8 + file).unwrap()
    }

    pub fn distance(self, other: Square) -> u8 {
        let file_diff = (self.file() as i8 - other.file() as i8).unsigned_abs();
        let rank_diff = (self.rank() as i8 - other.rank() as i8).unsigned_abs();
        file_diff + rank_diff
    }

    pub fn shift(self, offset: i8) -> Option<Square> {
        Square::from((self as i8 + offset) as u8)
    }
}

impl std::str::FromStr for Square {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let file = chars.next().ok_or(())?;
        let file = match file {
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

        let rank = chars.next().ok_or(())?.to_digit(10).ok_or(())?;
        if !(1..=8).contains(&rank) {
            return Err(());
        }

        Ok(Square::from(((rank - 1) * 8 + file) as u8).unwrap())
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let file = match (*self as u8) % 8 {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => unreachable!(),
        };

        let rank = (*self as u8) / 8 + 1;

        write!(f, "{}{}", file, rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn flip_square() {
        assert_eq!(Square::A1.flip(), Square::A8);
        assert_eq!(Square::A8.flip(), Square::A1);
        assert_eq!(Square::H1.flip(), Square::H8);
        assert_eq!(Square::H8.flip(), Square::H1);
        assert_eq!(Square::E4.flip(), Square::E5);
        assert_eq!(Square::E5.flip(), Square::E4);
        assert_eq!(Square::D3.flip(), Square::D6);
        assert_eq!(Square::D6.flip(), Square::D3);
    }

    #[test]
    fn fails_when_string_is_invalid() {
        assert!(Square::from_str("").is_err());
        assert!(Square::from_str("a").is_err());
        assert!(Square::from_str("a9").is_err());
        assert!(Square::from_str("i1").is_err());
    }

    #[test]
    fn parses_string() {
        assert_eq!(Square::from_str("a1"), Ok(Square::A1));
        assert_eq!(Square::from_str("a8"), Ok(Square::A8));
        assert_eq!(Square::from_str("h1"), Ok(Square::H1));
        assert_eq!(Square::from_str("h8"), Ok(Square::H8));
        assert_eq!(Square::from_str("e4"), Ok(Square::E4));
        assert_eq!(Square::from_str("d5"), Ok(Square::D5));
    }

    #[test]
    fn write_reflexive() {
        assert_eq!(Square::A1.to_string(), "a1");
        assert_eq!(Square::A8.to_string(), "a8");
        assert_eq!(Square::H1.to_string(), "h1");
        assert_eq!(Square::H8.to_string(), "h8");
        assert_eq!(Square::E4.to_string(), "e4");
        assert_eq!(Square::D5.to_string(), "d5");
    }
}
