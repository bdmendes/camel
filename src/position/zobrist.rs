use super::{Color, Piece, Position, Square, BOARD_SIZE};
use once_cell::sync::Lazy;
use rand::Rng;

pub type ZobristHash = u64;

static ZOBRIST_NUMBERS: Lazy<[ZobristHash; 12 * BOARD_SIZE]> = Lazy::new(|| {
    let mut rng = rand::thread_rng();
    let mut zobrist_numbers = [0; 12 * BOARD_SIZE];
    for i in 0..(12 * BOARD_SIZE) {
        zobrist_numbers[i] = rng.gen();
    }
    zobrist_numbers
});

fn zobrist_number(piece: Piece, square: Square) -> ZobristHash {
    let square_index = square.index;
    let index = piece as usize * 64 + square_index;
    ZOBRIST_NUMBERS[index]
}

pub fn zobrist_hash_position(position: &Position) -> ZobristHash {
    let mut hash: ZobristHash = 0;

    // Hash the pieces
    for index in 0..BOARD_SIZE {
        let square = Square { index };
        if let Some(piece) = position.at(square) {
            hash ^= zobrist_number(piece, square);
        }
    }

    // Reserve 1 + 4 + 6 = 11 bits for the next to move, castling rights and en passant square
    hash <<= 11;

    // Hash the active color
    if position.info.to_move == Color::Black {
        hash |= 0b1;
    }

    // Hash the castling rights
    hash |= (position.info.castling_rights.bits as ZobristHash) << 1;

    // Hash the en passant square
    if let Some(square) = position.info.en_passant_square {
        hash |= (square.index as ZobristHash) << 5;
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zobrist_same_hash() {
        let position = Position::new();
        let hash1 = zobrist_hash_position(&position);
        let hash2 = zobrist_hash_position(&position);
        assert_eq!(hash1, hash2);
    }
}
