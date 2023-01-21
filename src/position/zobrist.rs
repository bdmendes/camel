use super::{CastlingRights, Color, Piece, Position, Square, BOARD_SIZE};
use once_cell::sync::Lazy;
use rand::Rng;

static ZOBRIST_NUMBERS: Lazy<[u128; 12 * BOARD_SIZE as usize]> = Lazy::new(|| {
    let mut rng = rand::thread_rng();
    let mut zobrist_numbers = [0; 12 * BOARD_SIZE as usize];
    for i in 0..(12 * BOARD_SIZE as usize) {
        zobrist_numbers[i] = rng.gen();
    }
    zobrist_numbers
});

pub type ZobristHash = u128;

fn zobrist_number(piece: Piece<Color>, square: Square) -> u128 {
    let piece_index = match piece {
        Piece::Pawn(Color::White) => 0,
        Piece::Knight(Color::White) => 1,
        Piece::Bishop(Color::White) => 2,
        Piece::Rook(Color::White) => 3,
        Piece::Queen(Color::White) => 4,
        Piece::King(Color::White) => 5,
        Piece::Pawn(Color::Black) => 6,
        Piece::Knight(Color::Black) => 7,
        Piece::Bishop(Color::Black) => 8,
        Piece::Rook(Color::Black) => 9,
        Piece::Queen(Color::Black) => 10,
        Piece::King(Color::Black) => 11,
    };
    let square_index = square.index;
    let index = piece_index * 64 + square_index as usize;
    ZOBRIST_NUMBERS[index]
}

pub fn zobrist_hash_position(position: &Position) -> ZobristHash {
    let mut hash: ZobristHash = 0;

    // Hash the pieces
    for index in 0..BOARD_SIZE {
        let square = Square { index };
        if let Some(piece) = position.at(&square) {
            hash ^= zobrist_number(piece, square);
        }
    }

    // Reserve 1 + 4 + 6 = 11 bits for the next to move, castling rights and en passant square
    hash <<= 11;

    // Hash the active color
    if position.to_move == Color::Black {
        hash |= 0b1;
    }

    // Hash the castling rights
    hash |= (position.castling_rights.bits as ZobristHash) << 1;

    // Hash the en passant square
    if let Some(square) = position.en_passant_square {
        hash |= (square.index as ZobristHash) << 5;
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_position_same_hash() {
        let position = Position::new();
        let hash1 = zobrist_hash_position(&position);
        let hash2 = zobrist_hash_position(&position);
        assert_eq!(hash1, hash2);
    }
}
