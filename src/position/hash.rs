use std::{
    fs::File,
    io::{self, Write},
};

use ctor::ctor;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

use super::{
    bitboard::Bitboard,
    castling_rights::{CastlingRights, CastlingSide},
    color::Color,
    piece::Piece,
    square::Square,
};

#[derive(PartialEq, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ZobristHash(u64);

// 2 colors, 6 pieces, 64 squares + 1 color + 4 castling rights + 64 ep squares
const ZOBRIST_NUMBERS_SIZE: usize = 2 * 6 * 64 + 1 + 4 + 64;

#[ctor]
static ZOBRIST_NUMBERS: [ZobristHash; ZOBRIST_NUMBERS_SIZE] = unsafe {
    serde_json::from_str::<Vec<ZobristHash>>(include_str!("../../assets/dump/20260118-203201.zobrist"))
        .unwrap()
        .try_into()
        .unwrap()
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
        let idx = (color as usize) * 6 * 64 + (piece as usize) * 64 + square as usize;
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

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn ms32(&self) -> u32 {
        (self.0 >> 32) as u32
    }
}

pub fn gen_zobrist_numbers() -> Vec<ZobristHash> {
    let mut rng = StdRng::seed_from_u64(0);
    let mut numbers = vec![ZobristHash(0); ZOBRIST_NUMBERS_SIZE];
    numbers.iter_mut().for_each(|n| *n = ZobristHash(rng.next_u64()));
    numbers
}

pub fn save_zobrist_numbers(path: &str) -> io::Result<()> {
    let json = serde_json::to_string(&gen_zobrist_numbers()).unwrap();
    let mut output = File::create(path)?;
    output.write_all(json.as_bytes())
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::ZobristHash;
    use crate::position::{
        castling_rights::CastlingSide, color::Color, hash::save_zobrist_numbers, piece::Piece, square::Square,
    };
    use std::{collections::HashSet, io::BufReader};

    #[test]
    fn serialize_deserialize() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        save_zobrist_numbers(path).unwrap();
        let _: Vec<ZobristHash> = serde_json::from_reader(BufReader::new(file)).unwrap();
    }

    #[test]
    fn reflection() {
        let mut hash = ZobristHash(0);
        assert_eq!(hash.value(), 0);

        hash.xor_piece(Piece::Pawn, Square::E4, Color::White);
        assert_ne!(hash.value(), 0);
        hash.xor_piece(Piece::Pawn, Square::E4, Color::White);
        assert_eq!(hash.value(), 0);

        hash.xor_color();
        assert_ne!(hash.value(), 0);
        hash.xor_color();
        assert_eq!(hash.value(), 0);

        hash.xor_castle(Color::White, CastlingSide::Kingside);
        assert_ne!(hash.value(), 0);
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        assert_eq!(hash.value(), 0);

        hash.xor_ep_square(Square::E4);
        assert_ne!(hash.value(), 0);
        hash.xor_ep_square(Square::E4);
        assert_eq!(hash.value(), 0);

        hash.xor_color();
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        hash.xor_piece(Piece::King, Square::H8, Color::Black);
        hash.xor_color();
        hash.xor_castle(Color::White, CastlingSide::Kingside);
        hash.xor_piece(Piece::King, Square::H8, Color::Black);
        assert_eq!(hash.value(), 0);
    }

    #[test]
    fn piece_uniqueness() {
        let mut hash = ZobristHash(0);
        let mut seen = HashSet::new();

        for piece in Piece::list() {
            for color in Color::list() {
                for square in Square::list() {
                    hash.xor_piece(*piece, *square, *color);
                    assert!(!seen.contains(&hash.value()));
                    seen.insert(hash.value());
                    hash.xor_piece(*piece, *square, *color);
                }
            }
        }
    }

    #[test]
    fn minify() {
        assert_eq!(ZobristHash(u64::MAX).ms32(), u32::MAX);
        assert_eq!(ZobristHash((u32::MAX as u64) | (1u64 << 32)).ms32(), 1);
    }
}
