use self::{board::Board, square::Square};
use crate::moves::{
    gen::{checked_by, generate_moves, MoveStage},
    make_move, Move,
};
use bitflags::bitflags;
use ctor::ctor;
use primitive_enum::primitive_enum;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub mod bitboard;
pub mod board;
pub mod fen;
pub mod square;

pub type ZobristHash = u64;

// 2 colors, 6 pieces, 64 squares, 16 castling rights, 1 side to move, 64 en passant squares
const ZOBRIST_NUMBERS_SIZE: usize = 2 * 6 * 64 + 16 + 1 + 64;

#[ctor]
pub static ZOBRIST_NUMBERS: [ZobristHash; ZOBRIST_NUMBERS_SIZE] = {
    let mut rng = StdRng::seed_from_u64(0);
    let mut numbers = [0; ZOBRIST_NUMBERS_SIZE];
    numbers.iter_mut().take(ZOBRIST_NUMBERS_SIZE).for_each(|n| *n = rng.gen());
    numbers
};

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
    pub is_chess960: bool,
}

impl Position {
    pub fn zobrist_hash(&self) -> ZobristHash {
        let mut board_hash = self.board.zobrist_hash();
        board_hash ^= ZOBRIST_NUMBERS[2 * 6 * 64 + self.castling_rights.bits() as usize];
        if self.side_to_move == Color::Black {
            board_hash ^= ZOBRIST_NUMBERS[2 * 6 * 64 + 16];
        }
        if let Some(square) = self.en_passant_square {
            board_hash ^= ZOBRIST_NUMBERS[2 * 6 * 64 + 16 + 1 + square as usize];
        }
        board_hash
    }

    pub fn make_move(&self, mov: Move) -> Self {
        make_move::<true>(self, mov)
    }

    pub fn make_move_str(&self, mov_str: &str) -> Option<Self> {
        let moves = self.moves(MoveStage::All);
        let mov = moves.iter().find(|mov| mov.to_string() == mov_str)?;
        Some(self.make_move(*mov))
    }

    pub fn moves(&self, stage: MoveStage) -> Vec<Move> {
        generate_moves(stage, self)
    }

    pub fn is_check(&self) -> bool {
        checked_by(&self.board, self.side_to_move.opposite())
    }
}
