use num_enum::TryFromPrimitive;

#[rustfmt::skip]
#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Square {
    A1 = 0, B1 = 1, C1, D1, E1, F1, G1, H1,
    A2 = 8, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
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
        if rank < 1 || rank > 8 {
            return Err(());
        }

        Ok(Square::try_from(((rank - 1) * 8 + file) as u8).unwrap())
    }
}

impl ToString for Square {
    fn to_string(&self) -> String {
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

        format!("{}{}", file, rank)
    }
}
