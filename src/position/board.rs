use super::{bitboard::Bitboard, CastlingRights, Color, Square};
use ctor::ctor;
use primitive_enum::primitive_enum;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub type ZobristHash = u64;

// 2 colors, 6 pieces, 64 squares + 2 colors + 4 castling rights + 64 squares
const ZOBRIST_NUMBERS_SIZE: usize = 2 * 6 * 64 + 2 + 4 + 64;

#[ctor]
static ZOBRIST_NUMBERS: [ZobristHash; ZOBRIST_NUMBERS_SIZE] = {
    let mut rng = StdRng::seed_from_u64(0);
    let mut numbers = [0; ZOBRIST_NUMBERS_SIZE];
    numbers.iter_mut().take(ZOBRIST_NUMBERS_SIZE).for_each(|n| *n = rng.gen());
    numbers
};

primitive_enum!(
    Piece u8;
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    King
);

#[derive(Hash, PartialEq, Copy, Clone, Debug)]
pub struct Board {
    pieces: [Bitboard; 6],
    occupancy: [Bitboard; 2],
    mailbox: [Option<Piece>; 64],
    hash: ZobristHash,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            pieces: Default::default(),
            occupancy: Default::default(),
            hash: 0,
            mailbox: [None; 64],
        }
    }
}

impl Board {
    pub fn hash_color(color: Color) -> ZobristHash {
        ZOBRIST_NUMBERS[color as usize + 2 * 6 * 64]
    }

    pub fn hash_castling_rights(castling_rights: CastlingRights) -> ZobristHash {
        ZOBRIST_NUMBERS[2 * 6 * 64 + 2 + castling_rights.bits() as usize]
    }

    pub fn hash_enpassant(enpassant: Option<Square>) -> ZobristHash {
        ZOBRIST_NUMBERS[2 * 6 * 64 + 2 + 16 + enpassant.map(|s| s as usize).unwrap_or(0)]
    }

    pub fn zobrist_hash(&self) -> ZobristHash {
        self.hash
    }

    fn xor_hash(&mut self, square: Square, piece: Piece, color: Color) {
        let index = color as usize * 6 * 64 + piece as usize * 64 + square as usize;
        self.hash ^= ZOBRIST_NUMBERS[index];
    }

    pub fn set_square(&mut self, square: Square, piece: Piece, color: Color) {
        self.clear_square(square);
        self.pieces[piece as usize].set(square);
        self.occupancy[color as usize].set(square);
        self.mailbox[square as usize] = Some(piece);
        self.xor_hash(square, piece, color);
    }

    pub fn clear_square(&mut self, square: Square) {
        if let Some((piece, color)) = self.piece_color_at(square) {
            self.pieces[piece as usize].clear(square);
            self.occupancy[color as usize].clear(square);
            self.mailbox[square as usize] = None;
            self.xor_hash(square, piece, color);
        }
    }

    pub fn piece_color_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.piece_at(square).map(|piece| (piece, self.color_at(square).unwrap()))
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.mailbox[square as usize]
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        if self.occupancy[Color::White as usize].is_set(square) {
            Some(Color::White)
        } else if self.occupancy[Color::Black as usize].is_set(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn occupancy_bb_all(&self) -> Bitboard {
        self.occupancy[1] | self.occupancy[0]
    }

    pub fn occupancy_bb(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    pub fn pieces_bb(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize]
    }

    pub fn pieces_bb_color(&self, piece: Piece, color: Color) -> Bitboard {
        self.pieces[piece as usize] & self.occupancy[color as usize]
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
        board.mailbox[Square::E1 as usize] = Some(Piece::King);

        assert_eq!(board.piece_color_at(Square::E1), Some((Piece::King, Color::White)));
        assert_eq!(board.piece_color_at(Square::E2), None);

        assert_eq!(board.color_at(Square::E1), Some(Color::White));
        assert_eq!(board.color_at(Square::E2), None);
    }
}
