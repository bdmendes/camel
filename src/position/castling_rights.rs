use super::Color;

#[derive(PartialEq, Eq, Debug)]
pub struct CastlingRights(u8);

pub enum CastlingSide {
    Kingside,
    Queenside,
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
}

#[cfg(test)]
mod tests {
    use super::CastlingRights;
    use crate::position::{castling_rights::CastlingSide, Color};

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

        assert!(!castling_rights
            .removed_color(Color::White)
            .has_color(Color::White));
        assert!(!castling_rights
            .removed_color(Color::Black)
            .has_color(Color::Black));
    }
}
