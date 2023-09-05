use self::{
    board::Board,
    fen::{position_from_fen, position_to_fen},
    square::Square,
    zobrist::{ZobristHash, ZOBRIST_NUMBERS},
};
use crate::moves::{
    gen::{checked_by, generate_moves},
    make_move, Move,
};
use bitflags::bitflags;
use primitive_enum::primitive_enum;

pub mod bitboard;
pub mod board;
pub mod fen;
pub mod square;
pub mod zobrist;

primitive_enum!(
    Color u8;
    White,
    Black
);

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn sign(&self) -> i16 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

bitflags! {
    #[derive(Hash, PartialEq, Debug, Copy, Clone)]
    pub struct CastlingRights: u8 {
        const WHITE_KINGSIDE = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub board: Board,
    pub side_to_move: Color,
    pub en_passant_square: Option<Square>,
    pub castling_rights: CastlingRights,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Position {
    pub fn zobrist_hash(&self) -> ZobristHash {
        self.board.zobrist_hash()
            ^ ZOBRIST_NUMBERS[12 * 64 + self.side_to_move as usize]
            ^ ZOBRIST_NUMBERS[12 * 64 + 2 + self.en_passant_square.unwrap_or(Square::A1) as usize]
            ^ ZOBRIST_NUMBERS[12 * 64 + 2 + 64 + self.castling_rights.bits() as usize]
    }

    pub fn from_fen(fen: &str) -> Option<Position> {
        position_from_fen(fen)
    }

    pub fn to_fen(&self) -> String {
        position_to_fen(self)
    }

    pub fn make_move(&self, mov: Move) -> Self {
        make_move::<true>(self, mov)
    }

    pub fn moves<const QUIESCE: bool>(&self) -> Vec<Move> {
        generate_moves::<QUIESCE, false>(self)
    }

    pub fn is_check(&self) -> bool {
        checked_by(&self.board, self.side_to_move.opposite())
    }

    pub fn make_null_move(&self) -> Self {
        let mut position = *self;
        position.side_to_move = position.side_to_move.opposite();
        position
    }
}

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.zobrist_hash().hash(state);
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.zobrist_hash() == other.zobrist_hash()
    }
}

impl Eq for Position {}
