use std::fmt::{Display, Write};

use primitive_enum::primitive_enum;

static PIECE_VALUES: [i8; 6] = [1, 3, 3, 5, 9, 45];

primitive_enum! { Piece u8;
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn value(&self) -> i8 {
        PIECE_VALUES[*self as usize]
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        })
    }
}

impl TryFrom<char> for Piece {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c.to_ascii_lowercase() {
            'p' => Ok(Piece::Pawn),
            'n' => Ok(Piece::Knight),
            'b' => Ok(Piece::Bishop),
            'r' => Ok(Piece::Rook),
            'q' => Ok(Piece::Queen),
            'k' => Ok(Piece::King),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::piece::Piece;

    #[test]
    fn piece_value() {
        assert_eq!(Piece::Pawn.value(), 1);
        assert_eq!(Piece::Knight.value(), 3);
        assert_eq!(Piece::Bishop.value(), 3);
        assert_eq!(Piece::Rook.value(), 5);
        assert_eq!(Piece::Queen.value(), 9);
        assert_eq!(Piece::King.value(), 45);
    }
}
