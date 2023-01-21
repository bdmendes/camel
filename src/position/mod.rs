pub mod fen;
pub mod moves;
pub mod piece;
pub mod zobrist;

use bitflags::bitflags;
use std::fmt;

use self::fen::{position_from_fen, position_to_fen, START_FEN};
use self::moves::pseudo_legal_moves;
pub use self::piece::{Color, Piece};
use self::zobrist::ZobristHash;

pub const ROW_SIZE: u8 = 8;
pub const BOARD_SIZE: u8 = ROW_SIZE * ROW_SIZE;

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub struct Square {
    pub index: u8,
}

bitflags! {
    pub struct CastlingRights: u8 {
        const WHITE_KINGSIDE = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

pub struct Position {
    pub board: [Option<Piece<Color>>; 64], // 2D Little-Endian Rank-File Mapping
    pub to_move: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>,
    pub half_move_number: u8,
    pub full_move_number: u16,
}

impl Square {
    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push((('a' as u8) + (self.index as u8) % ROW_SIZE) as char);
        algebraic.push((('1' as u8) + (self.index as u8) / ROW_SIZE) as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: &str) -> Square {
        let mut chars = algebraic.chars();
        let col = chars.next().unwrap_or('a') as u8 - ('a' as u8);
        let row = chars.next().unwrap_or('1') as u8 - ('1' as u8);
        Square {
            index: row * ROW_SIZE as u8 + col,
        }
    }

    pub fn from_row_col(row: u8, col: u8) -> Square {
        Square {
            index: row * ROW_SIZE + col,
        }
    }

    pub fn row(&self) -> u8 {
        self.index / ROW_SIZE
    }

    pub fn col(&self) -> u8 {
        self.index % ROW_SIZE
    }
}

impl Position {
    pub fn at(&self, square: &Square) -> Option<Piece<Color>> {
        self.board[square.index as usize]
    }

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

    pub fn to_zobrist_hash(&self) -> ZobristHash {
        zobrist::zobrist_hash_position(&self)
    }

    pub fn is_check(&self, checked_player: Color, mid_castle_square: Option<Square>) -> bool {
        let opposing_color = checked_player.opposing();
        let opponent_moves = pseudo_legal_moves(&self, opposing_color);
        for move_ in opponent_moves {
            if let Some(Piece::King(color)) = self.at(&move_.to) {
                if color == checked_player {
                    return true;
                }
            }
            if let Some(mid_castle_square) = mid_castle_square {
                if move_.to == mid_castle_square {
                    return true;
                }
            }
        }
        false
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..8).rev() {
            for col in 0..8 {
                match self.board[(row * 8 + col) as usize] {
                    None => write!(f, " "),
                    Some(piece) => write!(f, "{}", piece.to_char()),
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
        assert_eq!(square.index, 0);

        let square = Square::from_row_col(7, 7);
        assert_eq!(square.index, 63);

        let square = Square::from_row_col(3, 4);
        assert_eq!(square.index, 28);

        let square = Square::from_row_col(4, 3);
        assert_eq!(square.index, 35);
    }

    #[test]
    fn square_from_algebraic() {
        let square = Square::from_algebraic("a1");
        assert_eq!(square.index, 0);

        let square = Square::from_algebraic("h8");
        assert_eq!(square.index, 63);

        let square = Square::from_algebraic("e4");
        assert_eq!(square.index, 28);

        let square = Square::from_algebraic("d5");
        assert_eq!(square.index, 35);
    }

    #[test]
    fn square_to_algebraic() {
        let square = Square { index: 0 };
        assert_eq!(square.to_algebraic(), "a1");

        let square = Square { index: 63 };
        assert_eq!(square.to_algebraic(), "h8");

        let square = Square { index: 28 };
        assert_eq!(square.to_algebraic(), "e4");

        let square = Square { index: 35 };
        assert_eq!(square.to_algebraic(), "d5");
    }

    #[test]
    fn square_row() {
        let square = Square { index: 0 };
        assert_eq!(square.row(), 0);

        let square = Square { index: 63 };
        assert_eq!(square.row(), 7);

        let square = Square { index: 28 };
        assert_eq!(square.row(), 3);

        let square = Square { index: 35 };
        assert_eq!(square.row(), 4);
    }

    #[test]
    fn square_col() {
        let square = Square { index: 0 };
        assert_eq!(square.col(), 0);

        let square = Square { index: 63 };
        assert_eq!(square.col(), 7);

        let square = Square { index: 28 };
        assert_eq!(square.col(), 4);

        let square = Square { index: 35 };
        assert_eq!(square.col(), 3);
    }
}
