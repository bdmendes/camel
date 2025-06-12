use self::{
    board::{Board, ZobristHash},
    square::Square,
};
use crate::{
    moves::{
        gen::{generate_moves, king_square_attackers, MoveStage},
        make_move, Move,
    },
    position::board::Piece,
};
use bitflags::bitflags;
use primitive_enum::primitive_enum;

pub mod bitboard;
pub mod board;
pub mod fen;
pub mod square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionDiffEntry {
    Set(Square, Piece, Color),
    Clear(Square, Piece, Color),
}

pub type PositionDiffVec = Vec<PositionDiffEntry>;

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
        self.board.zobrist_hash()
            ^ Board::hash_color(self.side_to_move)
            ^ Board::hash_castling_rights(self.castling_rights)
            ^ Board::hash_enpassant(self.en_passant_square)
    }

    pub fn make_move(&self, mov: Move) -> Self {
        make_move(self, mov)
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
        king_square_attackers::<true>(&self.board, self.side_to_move.opposite()).is_not_empty()
    }

    pub fn diff(&self, other: &Self) -> PositionDiffVec {
        let mut diff = PositionDiffVec::new();
        for square in Square::list() {
            let ours = self.board.piece_color_at(*square);
            let theirs = other.board.piece_color_at(*square);
            match (ours, theirs) {
                (Some((piece, color)), None) => {
                    diff.push(PositionDiffEntry::Set(*square, piece, color))
                }
                (None, Some((piece, color))) => {
                    diff.push(PositionDiffEntry::Clear(*square, piece, color))
                }
                (Some((piece1, color1)), Some((piece2, color2)))
                    if piece1 != piece2 || color1 != color2 =>
                {
                    diff.push(PositionDiffEntry::Set(*square, piece1, color1));
                    diff.push(PositionDiffEntry::Clear(*square, piece2, color2));
                }
                _ => {}
            }
        }
        diff
    }
}
