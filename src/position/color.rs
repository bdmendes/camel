use primitive_enum::primitive_enum;

primitive_enum! { Color u8;
    White,
    Black,
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

#[cfg(test)]
mod tests {
    use crate::position::color::Color;

    #[test]
    fn flipped() {
        assert_eq!(Color::White.flipped(), Color::Black);
        assert_eq!(Color::Black.flipped(), Color::White);
    }
}
