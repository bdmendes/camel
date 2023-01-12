mod fen;
mod moves;

use std::fmt;

use crate::position::fen::{position_from_fen, position_to_fen, START_FEN};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Square {
    pub row: u8,
    pub col: u8,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

pub struct Position {
    pub board: [[Option<(Piece, Color)>; 8]; 8], // 2D Little-Endian Rank-File Mapping
    pub next_to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub half_move_number: u8,
    pub full_move_number: u16,
}

impl Color {
    pub fn opposing(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Square {
    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push((('a' as u8) + self.col as u8) as char);
        algebraic.push((('1' as u8) + self.row as u8) as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: &str) -> Square {
        let mut chars = algebraic.chars();
        let col = chars.next().unwrap() as u8 - ('a' as u8);
        let row = (chars.next().unwrap() as u8 - ('1' as u8)) as u8;
        Square { row, col }
    }
}

impl Piece {
    pub fn from_char(c: char) -> Piece {
        match c {
            'p' => Piece::Pawn,
            'r' => Piece::Rook,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => panic!("Invalid piece"),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Rook => 'r',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }

    pub fn to_char_with_color(&self, color: Color) -> char {
        let c = self.to_char();
        match color {
            Color::White => c.to_uppercase().next().unwrap(),
            Color::Black => c,
        }
    }
}

impl Position {
    pub fn at(&self, square: &Square) -> Option<(Piece, Color)> {
        self.board[square.row as usize][square.col as usize]
    }

    pub fn new() -> Position {
        position_from_fen(START_FEN)
    }

    pub fn from_fen(fen: &str) -> Position {
        position_from_fen(fen)
    }

    pub fn to_fen(&self) -> String {
        position_to_fen(&self)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..8).rev() {
            for col in 0..8 {
                match self.board[row][col] {
                    None => write!(f, " "),
                    Some((piece, color)) => write!(f, "{}", piece.to_char_with_color(color)),
                }?;
            }
            if row > 0 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_square() {
        let square = Square::from_algebraic("a1");
        assert_eq!(square.to_algebraic(), "a1");
        assert_eq!(square.row, 0);
        assert_eq!(square.col, 0);

        let square = Square::from_algebraic("h8");
        assert_eq!(square.to_algebraic(), "h8");
        assert_eq!(square.row, 7);
        assert_eq!(square.col, 7);

        let square = Square::from_algebraic("e4");
        assert_eq!(square.to_algebraic(), "e4");
        assert_eq!(square.row, 3);
        assert_eq!(square.col, 4);

        let square = Square::from_algebraic("d5");
        assert_eq!(square.to_algebraic(), "d5");
        assert_eq!(square.row, 4);
        assert_eq!(square.col, 3);
    }
}
