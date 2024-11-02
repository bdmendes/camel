use bitboard::Bitboard;
use color::Color;
use hash::ZobristHash;
use square::Square;

mod bitboard;
mod castling_rights;
mod color;
mod hash;
mod square;

pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub struct Position {
    mailbox: [Option<Piece>; 64],
    pieces: [Bitboard; 6],
    occupancy: [Bitboard; 2],
    hash: ZobristHash,
    side_to_move: Color,
    ep_square: Option<Square>,
}
