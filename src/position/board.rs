use super::{
    bitboard::Bitboard,
    fen::board_from_fen,
    zobrist::{ZobristHash, ZOBRIST_NUMBERS},
    Color, Square,
};
use primitive_enum::primitive_enum;

primitive_enum!(
    Piece u8;
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    King
);

#[derive(Default, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Board {
    pieces: [Bitboard; 6],
    occupancy: [Bitboard; 2],
    hash: ZobristHash,
}

impl Board {
    pub fn new() -> Self {
        Board { pieces: Default::default(), occupancy: Default::default(), hash: 0 }
    }

    pub fn from_fen(board_fen: &str) -> Option<Board> {
        board_from_fen(board_fen)
    }

    pub fn zobrist_hash(&self) -> ZobristHash {
        self.hash
    }

    pub fn set_square<const CLEAR: bool>(&mut self, square: Square, piece: Piece, color: Color) {
        if CLEAR {
            self.clear_square(square);
        }
        self.pieces[piece as usize].set(square);
        self.occupancy[color as usize].set(square);

        self.hash ^=
            ZOBRIST_NUMBERS[(piece as usize * 64 + square as usize) + color as usize * 6 * 64];
    }

    pub fn clear_square(&mut self, square: Square) {
        if let Some((piece, color)) = self.piece_color_at(square) {
            self.pieces[piece as usize].clear(square);
            self.occupancy[color as usize].clear(square);

            self.hash ^=
                ZOBRIST_NUMBERS[(piece as usize * 64 + square as usize) + color as usize * 6 * 64];
        }
    }

    pub fn piece_color_at(&self, square: Square) -> Option<(Piece, Color)> {
        if let Some(color) = self.color_at(square) {
            return Some((self.piece_at(square).unwrap(), color));
        }
        None
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        for (piece, bb) in self.pieces.iter().enumerate() {
            if bb.is_set(square) {
                return Some(Piece::from(piece as u8).unwrap());
            }
        }
        None
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        if self.occupancy_bb(Color::White).is_set(square) {
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
        let pawns_bb = self.pieces_bb(Piece::Pawn) & self.occupancy_bb(color);
        for square in pawns_bb {
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
                let square = Square::from(rank * 8 + file).unwrap();
                let piece = self.piece_color_at(square);
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

        board.set_square::<true>(Square::E1, Piece::King, Color::White);

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

        assert_eq!(board.piece_color_at(Square::E1), Some((Piece::King, Color::White)));
        assert_eq!(board.piece_color_at(Square::E2), None);

        assert_eq!(board.color_at(Square::E1), Some(Color::White));
        assert_eq!(board.color_at(Square::E2), None);
    }
}
