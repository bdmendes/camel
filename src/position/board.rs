use num_enum::TryFromPrimitive;

use super::{bitboard::Bitboard, fen::board_from_fen, Color, Square};

pub const PIECES_NO_PAWN: [Piece; 5] =
    [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::King];
pub const PIECES: [Piece; 6] =
    [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Pawn, Piece::King];

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

#[derive(Default, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Board {
    pieces: [Bitboard; 6],
    occupancy: [Bitboard; 2],
}

impl Board {
    pub fn new(pieces: [Bitboard; 6], occupancy: [Bitboard; 2]) -> Self {
        Board { pieces, occupancy }
    }

    pub fn from_fen(board_fen: &str) -> Result<Board, ()> {
        board_from_fen(board_fen)
    }

    pub fn set_square(&mut self, square: Square, piece: Piece, color: Color) {
        self.clear_square(square);
        self.pieces[piece as usize].set(square);
        self.occupancy[color as usize].set(square);
    }

    pub fn clear_square(&mut self, square: Square) {
        for piece in &mut self.pieces {
            piece.clear(square);
        }
        for occupancy in &mut self.occupancy {
            occupancy.clear(square);
        }
    }

    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        let occupancy = self.occupancy_bb_all();
        if !occupancy.is_set(square) {
            return None;
        }

        let occupancy_white = self.occupancy_bb(Color::White);
        let occupancy_black = self.occupancy_bb(Color::Black);

        for (piece, bitboard) in self.pieces.iter().enumerate() {
            if bitboard.is_set(square) {
                let color = if occupancy_white.is_set(square) {
                    debug_assert!(!occupancy_black.is_set(square));
                    Color::White
                } else {
                    debug_assert!(occupancy_black.is_set(square));
                    Color::Black
                };
                return Some((Piece::try_from(piece as u8).unwrap(), color));
            }
        }
        None
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        if self.occupancy[Color::White as usize].is_set(square) {
            debug_assert!(!self.occupancy_bb(Color::Black).is_set(square));
            Some(Color::White)
        } else if self.occupancy_bb(Color::Black).is_set(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn occupancy_bb_all(&self) -> Bitboard {
        self.occupancy[0] | self.occupancy[1]
    }

    pub fn occupancy_bb(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    pub fn pieces_bb(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize]
    }

    pub fn piece_count(&self, color: Color) -> usize {
        let pieces_bb = self.pieces_bb(Piece::Queen)
            | self.pieces_bb(Piece::Rook)
            | self.pieces_bb(Piece::Bishop)
            | self.pieces_bb(Piece::Knight);
        let our_pieces_bb = pieces_bb & self.occupancy_bb(color);
        our_pieces_bb.count_ones() as usize
    }

    pub fn pawn_structure(&self, color: Color) -> [u8; 8] {
        let mut structure = [0; 8];
        let mut pawns_bb = self.pieces_bb(Piece::Pawn) & self.occupancy_bb(color);
        while let Some(square) = pawns_bb.pop_lsb() {
            structure[square.file() as usize] += 1;
        }
        structure
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut board = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::try_from(rank * 8 + file).unwrap();
                let piece = self.piece_at(square);
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
        let mut board = Board::default();

        board.set_square(Square::E1, Piece::King, Color::White);

        assert_eq!(*board.pieces[Piece::King as usize], 1 << Square::E1 as u8);
        assert_eq!(*board.occupancy[Color::White as usize], 1 << Square::E1 as u8);
        assert_eq!(*board.occupancy[Color::Black as usize], 0);

        board.clear_square(Square::E1);

        assert_eq!(*board.pieces[Piece::King as usize], 0);
        assert_eq!(*board.occupancy[Color::White as usize], 0);
        assert_eq!(*board.occupancy[Color::Black as usize], 0);
    }

    #[test]
    fn at() {
        let mut board = Board::default();

        *board.pieces[Piece::King as usize] = 1 << Square::E1 as u8;
        *board.occupancy[Color::White as usize] = 1 << Square::E1 as u8;

        assert_eq!(board.piece_at(Square::E1), Some((Piece::King, Color::White)));
        assert_eq!(board.piece_at(Square::E2), None);

        assert_eq!(board.color_at(Square::E1), Some(Color::White));
        assert_eq!(board.color_at(Square::E2), None);
    }
}