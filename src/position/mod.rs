pub mod board;
pub mod fen;
pub mod moves;
pub mod piece;
pub mod square;
pub mod zobrist;

use self::board::Board;
pub use self::piece::{Color, Piece};
use self::square::{Square, BOARD_SIZE};
use self::zobrist::ZobristHash;
use self::{
    fen::{position_from_fen, position_to_fen, START_FEN},
    moves::Move,
};
use bitflags::bitflags;
use std::fmt;

bitflags! {
    pub struct CastlingRights: u8 {
        const WHITE_KINGSIDE = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Position {
    pub board: Board, // 2D Little-Endian Rank-File Mapping
    pub to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub half_move_number: u8,
    pub full_move_number: u16,
}

impl Position {
    pub fn new() -> Position {
        position_from_fen(START_FEN).unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Position, String> {
        position_from_fen(fen)
    }

    #[allow(dead_code)]
    pub fn to_fen(&self) -> String {
        position_to_fen(&self, false)
    }

    pub fn to_fen_hash(&self) -> String {
        position_to_fen(&self, true)
    }

    pub fn zobrist_hash(&self) -> ZobristHash {
        zobrist::zobrist_hash_position(&self)
    }

    pub fn legal_moves(&self, only_non_quiet: bool) -> Vec<Move> {
        moves::legal_moves(&self, only_non_quiet)
    }

    pub fn make_move(&self, m: &moves::Move) -> Position {
        moves::make_move(&self, m)
    }

    pub fn make_null_move(&self) -> Position {
        moves::make_null_move(&self)
    }

    pub fn is_check(&self) -> bool {
        moves::position_is_check(&self, self.to_move, None)
    }

    pub fn piece_count(&self, color: Option<Color>, piece: Option<Piece>) -> usize {
        let mut count = 0;
        for square in 0..BOARD_SIZE {
            match self.board[square] {
                None => {}
                Some(Piece::WP) | Some(Piece::BP) | Some(Piece::WK) | Some(Piece::BK) => {}
                Some(p) => {
                    if color.is_none() || color.unwrap() == p.color() {
                        if piece.is_none() || piece.unwrap() == p {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..8).rev() {
            for col in 0..8 {
                match self.board[(row * 8 + col)] {
                    None => write!(f, "- "),
                    Some(piece) => write!(f, "{} ", piece.to_fancy_char()),
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
    fn square_from_row_col() {
        let square = Square::from_row_col(0, 0);
        assert_eq!(square.0, 0);

        let square = Square::from_row_col(7, 7);
        assert_eq!(square.0, 63);

        let square = Square::from_row_col(3, 4);
        assert_eq!(square.0, 28);

        let square = Square::from_row_col(4, 3);
        assert_eq!(square.0, 35);
    }

    #[test]
    fn square_from_algebraic() {
        let square = Square::from_algebraic("a1");
        assert_eq!(square.0, 0);

        let square = Square::from_algebraic("h8");
        assert_eq!(square.0, 63);

        let square = Square::from_algebraic("e4");
        assert_eq!(square.0, 28);

        let square = Square::from_algebraic("d5");
        assert_eq!(square.0, 35);
    }

    #[test]
    fn square_to_algebraic() {
        let square = Square(0);
        assert_eq!(square.to_algebraic(), "a1");

        let square = Square(63);
        assert_eq!(square.to_algebraic(), "h8");

        let square = Square(28);
        assert_eq!(square.to_algebraic(), "e4");

        let square = Square(35);
        assert_eq!(square.to_algebraic(), "d5");
    }

    #[test]
    fn square_row() {
        let square = Square(0);
        assert_eq!(square.row(), 0);

        let square = Square(63);
        assert_eq!(square.row(), 7);

        let square = Square(28);
        assert_eq!(square.row(), 3);

        let square = Square(35);
        assert_eq!(square.row(), 4);
    }

    #[test]
    fn square_col() {
        let square = Square(0);
        assert_eq!(square.col(), 0);

        let square = Square(63);
        assert_eq!(square.col(), 7);

        let square = Square(28);
        assert_eq!(square.col(), 4);

        let square = Square(35);
        assert_eq!(square.col(), 3);
    }
}
