use std::fmt::{Display, Write};

use primitive_enum::primitive_enum;

static FLIPPED_COLORS: [Color; 2] = [Color::Black, Color::White];
static COLOR_SIGNS: [i8; 2] = [1, -1];

primitive_enum! { Color u8;
    White,
    Black
}

impl Color {
    pub fn flipped(self) -> Self {
        FLIPPED_COLORS[self as usize]
    }

    pub fn sign(self) -> i8 {
        COLOR_SIGNS[self as usize]
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

#[cfg(test)]
mod tests {
    use crate::core::color::Color;

    #[test]
    fn flip() {
        assert_eq!(Color::White.flipped(), Color::Black);
        assert_eq!(Color::Black.flipped(), Color::White);
    }

    #[test]
    fn sign() {
        assert_eq!(Color::White.sign(), 1);
        assert_eq!(Color::Black.sign(), -1);
    }
}
