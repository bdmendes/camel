pub type Bitboard = u64;

enum Piece {
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    King,
}

pub struct Board {
    pub pieces: [Bitboard; 6],
    pub occupancy: [Bitboard; 2],
}
