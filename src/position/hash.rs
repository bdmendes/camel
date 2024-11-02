use super::{
    square::{self, Square},
    Piece,
};

#[derive(PartialEq, Eq, Debug, Default)]
pub struct ZobristHash(u64);

impl ZobristHash {
    pub fn xor_piece(&mut self, piece: Piece, square: Square) {}
}
