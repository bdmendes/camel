use self::{
    board::Board,
    fen::{position_from_fen, position_to_fen},
    square::Square,
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
    pub fn from_fen(fen: &str) -> Option<Position> {
        position_from_fen(fen)
    }

    pub fn to_fen(&self) -> String {
        position_to_fen(self)
    }

    pub fn make_move(&self, mov: Move) -> Self {
        make_move(self, mov)
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
        self.board.hash(state);
        self.side_to_move.hash(state);
        self.en_passant_square.hash(state);
        self.castling_rights.hash(state);
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.side_to_move == other.side_to_move
            && self.en_passant_square == other.en_passant_square
            && self.castling_rights == other.castling_rights
    }
}

impl Eq for Position {}
