use num_enum::TryFromPrimitive;

use super::{Color, Square};

pub type Bitboard = u64;

#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Piece {
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    King,
}

#[derive(Hash, PartialEq)]
pub struct Board {
    pub pieces: [Bitboard; 6],
    pub occupancy: [Bitboard; 2],
}

impl Board {
    pub fn new() -> Self {
        Board { pieces: [0; 6], occupancy: [0; 2] }
    }

    pub fn set_square(&mut self, square: Square, piece: Piece, color: Color) {
        self.clear_square(square);
        self.pieces[piece as usize] |= 1 << (square as u8);
        self.occupancy[color as usize] |= 1 << (square as u8);
    }

    pub fn clear_square(&mut self, square: Square) {
        for piece in &mut self.pieces {
            *piece &= !(1 << (square as u8));
        }
        for occupancy in &mut self.occupancy {
            *occupancy &= !(1 << (square as u8));
        }
    }

    pub fn at(&self, square: Square) -> Option<(Piece, Color)> {
        for (piece, bitboard) in self.pieces.iter().enumerate() {
            if bitboard & (1 << (square as u8)) != 0 {
                let color = if self.occupancy[Color::White as usize] & (1 << (square as u8)) != 0 {
                    debug_assert!(
                        self.occupancy[Color::Black as usize] & (1 << (square as u8)) == 0
                    );
                    Color::White
                } else {
                    debug_assert!(
                        self.occupancy[Color::Black as usize] & (1 << (square as u8)) != 0
                    );
                    Color::Black
                };
                return Some((Piece::try_from(piece as u8).unwrap(), color));
            }
        }
        None
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::try_from(rank * 8 + file).unwrap();
                let piece = self.at(square);
                board.push(match piece {
                    Some((Piece::King, Color::White)) => '♔',
                    Some((Piece::Queen, Color::White)) => '♕',
                    Some((Piece::Rook, Color::White)) => '♖',
                    Some((Piece::Bishop, Color::White)) => '♗',
                    Some((Piece::Knight, Color::White)) => '♘',
                    Some((Piece::Pawn, Color::White)) => '♙',
                    Some((Piece::King, Color::Black)) => '♚',
                    Some((Piece::Queen, Color::Black)) => '♛',
                    Some((Piece::Rook, Color::Black)) => '♜',
                    Some((Piece::Bishop, Color::Black)) => '♝',
                    Some((Piece::Knight, Color::Black)) => '♞',
                    Some((Piece::Pawn, Color::Black)) => '♟',
                    None => '-',
                });
                board.push(' ');
            }
            board.push('\n');
        }
        write!(f, "{}", board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_clear() {
        let mut board = Board::new();

        board.set_square(Square::E1, Piece::King, Color::White);

        assert_eq!(board.pieces[Piece::King as usize], 1 << Square::E1 as u8);
        assert_eq!(board.occupancy[Color::White as usize], 1 << Square::E1 as u8);
        assert_eq!(board.occupancy[Color::Black as usize], 0);

        board.clear_square(Square::E1);

        assert_eq!(board.pieces[Piece::King as usize], 0);
        assert_eq!(board.occupancy[Color::White as usize], 0);
        assert_eq!(board.occupancy[Color::Black as usize], 0);
    }

    #[test]
    fn at() {
        let mut board = Board::new();

        board.pieces[Piece::King as usize] = 1 << Square::E1 as u8;
        board.occupancy[Color::White as usize] = 1 << Square::E1 as u8;

        assert_eq!(board.at(Square::E1), Some((Piece::King, Color::White)));
        assert_eq!(board.at(Square::E2), None);
    }
}
