use std::fmt::{Display, Write};

use super::color::Color;
use primitive_enum::primitive_enum;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct CastlingRights(u8);

primitive_enum! { CastlingSide u8;
    Kingside,
    Queenside,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::new(true, true, true, true)
    }
}

impl CastlingRights {
    pub fn new(
        white_kingside: bool,
        white_queenside: bool,
        black_kingside: bool,
        black_queenside: bool,
    ) -> Self {
        let mut payload = 0;
        white_kingside.then(|| payload |= 0b1);
        white_queenside.then(|| payload |= 0b10);
        black_kingside.then(|| payload |= 0b100);
        black_queenside.then(|| payload |= 0b1000);
        Self(payload)
    }

    fn mask_color(color: Color) -> &'static u8 {
        match color {
            Color::White => &0b11,
            Color::Black => &0b1100,
        }
    }

    fn mask_side(color: Color, side: CastlingSide) -> &'static u8 {
        match (color, side) {
            (Color::White, CastlingSide::Kingside) => &0b1,
            (Color::White, CastlingSide::Queenside) => &0b10,
            (Color::Black, CastlingSide::Kingside) => &0b100,
            (Color::Black, CastlingSide::Queenside) => &0b1000,
        }
    }

    pub fn has_color(&self, color: Color) -> bool {
        (self.0 & Self::mask_color(color)) != 0
    }

    pub fn has_side(&self, color: Color, side: CastlingSide) -> bool {
        (self.0 & Self::mask_side(color, side)) != 0
    }

    pub fn removed_color(&self, color: Color) -> Self {
        Self(self.0 & !Self::mask_color(color))
    }

    pub fn removed_side(&self, color: Color, side: CastlingSide) -> Self {
        Self(self.0 & !Self::mask_side(color, side))
    }

    pub fn xor(&self, other: CastlingRights) -> Self {
        Self(self.0 ^ other.0)
    }

    pub fn reversed(&self) -> Self {
        Self((!self.0) & 0b1111)
    }
}

impl Iterator for CastlingRights {
    type Item = (Color, CastlingSide);

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let lsb = self.0.trailing_zeros();
        self.0 &= self.0 - 1;
        Some(match lsb {
            0 => (Color::White, CastlingSide::Kingside),
            1 => (Color::White, CastlingSide::Queenside),
            2 => (Color::Black, CastlingSide::Kingside),
            _ => (Color::Black, CastlingSide::Queenside),
        })
    }
}

impl Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == 0 {
            return f.write_char('-');
        }

        for (color, side) in self.into_iter() {
            f.write_char(match (color, side) {
                (Color::White, CastlingSide::Kingside) => 'K',
                (Color::White, CastlingSide::Queenside) => 'Q',
                (Color::Black, CastlingSide::Kingside) => 'k',
                (Color::Black, CastlingSide::Queenside) => 'q',
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CastlingRights;
    use crate::core::position::{castling_rights::CastlingSide, color::Color};

    #[test]
    fn has() {
        let castling_rights = CastlingRights::new(true, false, false, true);

        assert!(castling_rights.has_color(Color::White));
        assert!(castling_rights.has_color(Color::Black));

        assert!(castling_rights.has_side(Color::White, CastlingSide::Kingside));
        assert!(!castling_rights.has_side(Color::White, CastlingSide::Queenside));

        assert!(!castling_rights.has_side(Color::Black, CastlingSide::Kingside));
        assert!(castling_rights.has_side(Color::Black, CastlingSide::Queenside));
    }

    #[test]
    fn removed() {
        let castling_rights = CastlingRights::new(true, false, false, true);

        assert_ne!(
            castling_rights,
            castling_rights.removed_side(Color::White, CastlingSide::Kingside)
        );
        assert_eq!(
            castling_rights,
            castling_rights.removed_side(Color::White, CastlingSide::Queenside)
        );
        assert_eq!(
            castling_rights,
            castling_rights.removed_side(Color::Black, CastlingSide::Kingside)
        );
        assert_ne!(
            castling_rights,
            castling_rights.removed_side(Color::Black, CastlingSide::Queenside)
        );

        assert!(
            !castling_rights
                .removed_color(Color::White)
                .has_color(Color::White)
        );
        assert!(
            !castling_rights
                .removed_color(Color::Black)
                .has_color(Color::Black)
        );
    }

    #[test]
    fn xor() {
        let castling_rights1 = CastlingRights::new(true, false, false, true);
        let castling_rights2 = CastlingRights::new(true, true, false, false);
        let castling_rights3 = CastlingRights::new(false, true, false, true);
        assert_eq!(castling_rights1.xor(castling_rights2), castling_rights3);
    }

    #[test]
    fn reversed() {
        let castling_rights1 = CastlingRights::new(true, false, false, true);
        let castling_rights2 = CastlingRights::new(false, true, true, false);
        assert_eq!(castling_rights1.reversed(), castling_rights2);
    }

    #[test]
    fn iter() {
        let castling_rights = CastlingRights::new(true, true, false, true);
        let mut iter = castling_rights.into_iter();
        assert_eq!(iter.next(), Some((Color::White, CastlingSide::Kingside)));
        assert_eq!(iter.next(), Some((Color::White, CastlingSide::Queenside)));
        assert_eq!(iter.next(), Some((Color::Black, CastlingSide::Queenside)));
        assert_eq!(iter.next(), None);
    }
}
