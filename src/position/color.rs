use std::fmt::{Display, Write};

use primitive_enum::primitive_enum;

primitive_enum! { Color u8;
    White,
    Black
}

impl Color {
    pub fn flipped(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn sign(self) -> &'static i8 {
        match self {
            Color::White => &1,
            Color::Black => &-1,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Color::White => 'w',
            Color::Black => 'b',
        })
    }
}
