use ctor::ctor;
use rand::{rngs::StdRng, Rng, SeedableRng};

use super::{
    bitboard::Bitboard,
    castling_rights::{CastlingRights, CastlingSide},
    color::Color,
    piece::Piece,
    square::Square,
};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct ZobristHash(u64);

// 2 colors, 6 pieces, 64 squares + 1 color + 4 castling rights + 64 ep squares
const ZOBRIST_NUMBERS_SIZE: usize = 2 * 6 * 64 + 2 + 4 + 64;

#[ctor]
static ZOBRIST_NUMBERS: [ZobristHash; ZOBRIST_NUMBERS_SIZE] = {
    let mut rng = StdRng::seed_from_u64(0);
    let mut numbers = [0; ZOBRIST_NUMBERS_SIZE];
    numbers
        .iter_mut()
        .take(ZOBRIST_NUMBERS_SIZE)
        .for_each(|n| *n = rng.gen());
    numbers.map(ZobristHash)
};

impl ZobristHash {
    pub fn new(
        pieces: [Bitboard; 6],
        occupancy: [Bitboard; 2],
        side_to_move: Color,
        castling_rights: CastlingRights,
        ep_square: Option<Square>,
    ) -> Self {
        let mut hash = Self(0);

        let occupancy_all = occupancy[0] | occupancy[1];
        for square in occupancy_all {
            let piece = pieces
                .iter()
                .position(|bb| bb.is_set(square))
                .map(|idx| Piece::from(idx as u8).unwrap())
                .unwrap();
            hash.xor_piece(
                piece,
                square,
                if occupancy[Color::White as usize].is_set(square) {
                    Color::White
                } else {
                    Color::Black
                },
            );
        }

        if side_to_move == Color::Black {
            hash.xor_color();
        }

        for side in CastlingSide::list() {
            for color in Color::list() {
                if castling_rights.has_side(*color, *side) {
                    hash.xor_castle(*color, *side);
                }
            }
        }

        if let Some(ep_square) = ep_square {
            hash.xor_ep_square(ep_square);
        }

        hash
    }

    pub fn xor_piece(&mut self, piece: Piece, square: Square, color: Color) {
        let idx = (color as usize) * (piece as usize) + square as usize;
        self.0 ^= ZOBRIST_NUMBERS[idx].0;
    }

    pub fn xor_color(&mut self) {
        self.0 ^= ZOBRIST_NUMBERS[2 * 6 * 64].0;
    }

    pub fn xor_castle(&mut self, color: Color, side: CastlingSide) {
        let offset = match (color, side) {
            (Color::White, CastlingSide::Kingside) => 0,
            (Color::White, CastlingSide::Queenside) => 1,
            (Color::Black, CastlingSide::Kingside) => 2,
            (Color::Black, CastlingSide::Queenside) => 3,
        };
        self.0 ^= ZOBRIST_NUMBERS[2 * 6 * 64 + 1 + offset].0;
    }

    pub fn xor_ep_square(&mut self, square: Square) {
        self.0 ^= ZOBRIST_NUMBERS[2 * 6 * 64 + 1 + 4 + square as usize].0;
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{castling_rights::CastlingSide, color::Color, piece::Piece, square::Square};

    use super::ZobristHash;

    #[test]
    fn reflection() {
        let mut hash = ZobristHash(0);
        assert_eq!(hash.0, 0);

        hash.xor_piece(Piece::Pawn, Square::E4, Color::White);
        assert_ne!(hash.0, 0);
        hash.xor_piece(Piece::Pawn, Square::E4, Color::White);
        assert_eq!(hash.0, 0);

        hash.xor_color();
        assert_ne!(hash.0, 0);
        hash.xor_color();
        assert_eq!(hash.0, 0);

        hash.xor_castle(Color::White, CastlingSide::Kingside);
        assert_ne!(hash.0, 0);
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        assert_eq!(hash.0, 0);

        hash.xor_ep_square(Square::E4);
        assert_ne!(hash.0, 0);
        hash.xor_ep_square(Square::E4);
        assert_eq!(hash.0, 0);

        hash.xor_color();
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        hash.xor_piece(Piece::King, Square::H8, Color::Black);
        hash.xor_color();
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        hash.xor_piece(Piece::King, Square::H8, Color::Black);
        assert_eq!(hash.0, 0);
    }
}
