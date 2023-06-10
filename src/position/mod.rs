use bitflags::bitflags;
use num_enum::TryFromPrimitive;

use self::{
    board::Board,
    fen::{position_from_fen, position_to_fen},
    square::Square,
};

pub mod board;
pub mod fen;
pub mod square;

#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

bitflags! {
    #[derive(Hash, PartialEq, Debug)]
    pub struct CastlingRights: u8 {
        const WHITE_KINGSIDE = 0b0001;
        const WHITE_QUEENSIDE = 0b0010;
        const BLACK_KINGSIDE = 0b0100;
        const BLACK_QUEENSIDE = 0b1000;
    }
}

pub struct Position {
    board: Board,
    side_to_move: Color,
    en_passant_square: Option<Square>,
    castling_rights: CastlingRights,
    halfmove_clock: u8,
    fullmove_number: u16,
}

impl Position {
    pub fn from_fen(fen: &str) -> Result<Position, ()> {
        position_from_fen(fen)
    }

    pub fn to_fen(&self) -> String {
        position_to_fen(self)
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
