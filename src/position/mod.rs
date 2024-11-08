use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use bitboard::Bitboard;
use castling_rights::CastlingRights;
use color::Color;
use fen::Fen;
use hash::ZobristHash;
use piece::Piece;
use square::Square;

pub mod bitboard;
pub mod castling_rights;
pub mod color;
pub mod fen;
pub mod hash;
pub mod piece;
pub mod square;

#[derive(Debug, Clone)]
pub struct Position {
    hash: ZobristHash,
    pieces: [Bitboard; 6],
    occupancy: [Bitboard; 2],
    side_to_move: Color,
    ep_square: Option<Square>,
    castling_rights: CastlingRights,
    halfmove_clock: u8,
    fullmove_number: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            hash: ZobristHash::new(
                [Bitboard::default(); 6],
                [Bitboard::default(); 2],
                Color::White,
                CastlingRights::default(),
                None,
            ),
            pieces: [Bitboard::default(); 6],
            occupancy: [Bitboard::default(); 2],
            side_to_move: Color::White,
            ep_square: None,
            castling_rights: CastlingRights::default(),
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Position {}

impl FromStr for Position {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Position::try_from(Fen::from_str(s).unwrap())
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..=7).rev() {
            for file in 0..=7 {
                let square = Square::from_file_rank(file, rank).unwrap();
                if let Some((piece, color)) = self.piece_color_at(square) {
                    f.write_char(match (piece, color) {
                        (Piece::Pawn, Color::Black) => '♙',
                        (Piece::Pawn, Color::White) => '♟',
                        (Piece::Knight, Color::Black) => '♘',
                        (Piece::Knight, Color::White) => '♞',
                        (Piece::Bishop, Color::Black) => '♗',
                        (Piece::Bishop, Color::White) => '♝',
                        (Piece::Rook, Color::Black) => '♖',
                        (Piece::Rook, Color::White) => '♜',
                        (Piece::Queen, Color::Black) => '♕',
                        (Piece::Queen, Color::White) => '♛',
                        (Piece::King, Color::Black) => '♔',
                        (Piece::King, Color::White) => '♚',
                    })?;
                } else {
                    f.write_char('_')?;
                }
                f.write_char(' ')?;
            }
            f.write_char('\n')?;
        }
        f.write_str(&format!("{}\n", Fen::from(self)))
    }
}

impl Position {
    pub fn occupancy_bb(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    pub fn occupancy_bb_all(&self) -> Bitboard {
        self.occupancy[0] | self.occupancy[1]
    }

    pub fn pieces_bb(&self, piece: Piece) -> Bitboard {
        self.pieces[piece as usize]
    }

    pub fn color_at(&self, square: Square) -> Option<Color> {
        if self.occupancy[0].is_set(square) {
            Some(Color::White)
        } else if self.occupancy[1].is_set(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces
            .iter()
            .position(|bb| bb.is_set(square))
            .map(|idx| Piece::from(idx as u8).unwrap())
    }

    pub fn piece_color_at(&self, square: Square) -> Option<(Piece, Color)> {
        self.color_at(square)
            .map(|c| (self.piece_at(square).unwrap(), c))
    }

    pub fn clear_square(&mut self, square: Square) {
        if let Some((piece, color)) = self.piece_color_at(square) {
            self.pieces[piece as usize].clear(square);
            self.occupancy[color as usize].clear(square);
            self.hash.xor_piece(piece, square, color);
        }
    }

    pub fn set_square(&mut self, square: Square, piece: Piece, color: Color) {
        self.clear_square(square);
        self.pieces[piece as usize].set(square);
        self.occupancy[color as usize].set(square);
        self.hash.xor_piece(piece, square, color);
    }

    pub fn hash(&self) -> ZobristHash {
        self.hash
    }

    pub fn hash_from_scratch(&self) -> ZobristHash {
        ZobristHash::new(
            self.pieces,
            self.occupancy,
            self.side_to_move,
            self.castling_rights,
            self.ep_square,
        )
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn flip_side_to_move(&mut self) {
        self.side_to_move = self.side_to_move.flipped();
        self.hash.xor_color();
    }

    pub fn ep_square(&self) -> Option<Square> {
        self.ep_square
    }

    pub fn clear_ep_square(&mut self) {
        if let Some(ep_square) = self.ep_square {
            self.ep_square = None;
            self.hash.xor_ep_square(ep_square);
        }
    }

    pub fn set_ep_square(&mut self, ep_square: Square) {
        self.clear_ep_square();
        self.ep_square = Some(ep_square);
        self.hash.xor_ep_square(ep_square);
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        for (color, side) in self.castling_rights.xor(castling_rights) {
            self.hash.xor_castle(color, side);
        }
        self.castling_rights = castling_rights;
    }

    pub fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    pub fn set_halfmove_clock(&mut self, halfmove_clock: u8) {
        self.halfmove_clock = halfmove_clock;
    }

    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    pub fn set_fullmove_number(&mut self, fullmove_number: u16) {
        self.fullmove_number = fullmove_number;
    }
}

#[cfg(test)]
mod tests {
    use crate::position::{castling_rights::CastlingSide, color::Color, square::Square, Piece};

    use super::Position;

    #[test]
    fn pieces() {
        let mut position = Position::default();
        let hash1 = position.hash();

        assert_eq!(position.piece_at(Square::E4), None);
        assert_eq!(position.color_at(Square::E4), None);
        assert_eq!(position.piece_color_at(Square::E4), None);

        position.set_square(Square::E4, Piece::Pawn, Color::White);
        let hash2 = position.hash();
        assert_eq!(position.piece_at(Square::E4), Some(Piece::Pawn));
        assert_eq!(position.color_at(Square::E4), Some(Color::White));
        assert_eq!(
            position.piece_color_at(Square::E4),
            Some((Piece::Pawn, Color::White))
        );
        assert_ne!(hash1, hash2);

        position.clear_square(Square::E4);
        let hash3 = position.hash();
        assert_eq!(position.piece_at(Square::E4), None);
        assert_eq!(position.color_at(Square::E4), None);
        assert_eq!(position.piece_color_at(Square::E4), None);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn side_to_move() {
        let mut position = Position::default();
        let hash1 = position.hash();
        assert_eq!(position.side_to_move(), Color::White);

        position.flip_side_to_move();
        let hash2 = position.hash();
        assert_ne!(hash1, hash2);
        assert_eq!(position.side_to_move(), Color::Black);

        position.flip_side_to_move();
        let hash3 = position.hash();
        assert_eq!(position.side_to_move(), Color::White);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn ep_square() {
        let mut position = Position::default();
        let hash1 = position.hash();
        assert_eq!(position.ep_square(), None);

        position.set_ep_square(Square::C6);
        let hash2 = position.hash();
        assert_eq!(position.ep_square(), Some(Square::C6));
        assert_ne!(hash1, hash2);

        position.set_ep_square(Square::B6);
        let hash3 = position.hash();
        assert_eq!(position.ep_square(), Some(Square::B6));
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);

        position.clear_ep_square();
        let hash4 = position.hash();
        assert_eq!(position.ep_square(), None);
        assert_eq!(hash4, hash1);
    }

    #[test]
    fn castling_rights() {
        let mut position = Position::default();
        let hash1 = position.hash();
        assert!(position.castling_rights().has_color(Color::White));
        assert!(position.castling_rights().has_color(Color::Black));
        assert!(position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Kingside));
        assert!(position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Queenside));
        assert!(position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Kingside));
        assert!(position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Queenside));

        position.set_castling_rights(position.castling_rights().removed_color(Color::White));
        let hash2 = position.hash();
        assert!(!position.castling_rights().has_color(Color::White));
        assert!(position.castling_rights().has_color(Color::Black));
        assert!(!position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Kingside));
        assert!(!position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Queenside));
        assert!(position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Kingside));
        assert!(position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Queenside));
        assert_ne!(hash1, hash2);

        position.set_castling_rights(
            position
                .castling_rights()
                .removed_side(Color::Black, CastlingSide::Kingside),
        );
        let hash3 = position.hash();
        assert!(!position.castling_rights().has_color(Color::White));
        assert!(position.castling_rights().has_color(Color::Black));
        assert!(!position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Kingside));
        assert!(!position
            .castling_rights()
            .has_side(Color::White, CastlingSide::Queenside));
        assert!(!position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Kingside));
        assert!(position
            .castling_rights()
            .has_side(Color::Black, CastlingSide::Queenside));
        assert_ne!(hash2, hash3);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn hash_validity() {
        let mut position = Position::default();
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_square(Square::E4, Piece::Pawn, Color::White);
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_square(Square::D5, Piece::Knight, Color::Black);
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.clear_square(Square::E4);
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.flip_side_to_move();
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_castling_rights(position.castling_rights.removed_color(Color::White));
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_castling_rights(
            position
                .castling_rights
                .removed_side(Color::White, CastlingSide::Kingside),
        );
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_castling_rights(
            position
                .castling_rights
                .removed_side(Color::Black, CastlingSide::Kingside),
        );
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.set_ep_square(Square::C6);
        assert_eq!(position.hash(), position.hash_from_scratch());

        position.clear_ep_square();
        assert_eq!(position.hash(), position.hash_from_scratch());
    }
}
