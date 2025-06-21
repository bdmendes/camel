use derive_more::derive::FromStr;
use std::{fmt::Display, str::FromStr};

use super::{
    Position,
    castling_rights::{CastlingRights, CastlingSide},
    color::Color,
    piece::Piece,
    square::Square,
};

pub const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

#[derive(PartialEq, Eq, Debug, Clone, FromStr)]
pub struct Fen(String);

impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&Position> for Fen {
    fn from(position: &Position) -> Self {
        let mut str = String::new();
        let mut empty_spaces = 0;

        for rank in (0..=7).rev() {
            for file in 0..=7 {
                let square = Square::from_file_rank(file, rank).unwrap();
                if let Some((piece, color)) = position.piece_color_at(square) {
                    if empty_spaces > 0 {
                        str.push_str(&empty_spaces.to_string());
                    }
                    str.push_str(&match color {
                        Color::White => piece.to_string().to_uppercase(),
                        Color::Black => piece.to_string(),
                    });
                    empty_spaces = 0;
                } else {
                    empty_spaces += 1;
                }
            }
            if empty_spaces != 0 {
                str.push_str(&empty_spaces.to_string());
                empty_spaces = 0;
            }
            if rank > 0 {
                str.push('/');
            }
        }

        str.push(' ');
        str.push_str(&position.side_to_move().to_string());

        str.push(' ');
        str.push_str(&position.castling_rights().to_string());

        str.push(' ');
        str.push_str(&match position.ep_square() {
            Some(square) => square.to_string(),
            None => "-".to_string(),
        });

        str.push(' ');
        str.push_str(&position.halfmove_clock().to_string());

        str.push(' ');
        str.push_str(&position.fullmove_number().to_string());

        Self(str)
    }
}

impl TryFrom<Fen> for Position {
    type Error = ();

    fn try_from(fen: Fen) -> Result<Self, ()> {
        fn mark_960(position: &mut Position, castling_side: CastlingSide, color: Color) {
            let rook = match (castling_side, color) {
                (CastlingSide::Kingside, Color::White) => Square::H1,
                (CastlingSide::Kingside, Color::Black) => Square::H8,
                (CastlingSide::Queenside, Color::White) => Square::A1,
                (CastlingSide::Queenside, Color::Black) => Square::A8,
            };
            let king = match color {
                Color::White => Square::E1,
                Color::Black => Square::E8,
            };
            let is_chess960 = !position.pieces_color_bb(Piece::Rook, color).is_set(rook)
                || !position.pieces_color_bb(Piece::King, color).is_set(king);
            if is_chess960 {
                position.chess960 = true;
            }
        }

        let mut position = Position::default();
        let mut words = fen.0.split_whitespace();
        let mut rank: u8 = 7;
        let mut file: u8 = 0;

        for c in words.next().ok_or(())?.chars() {
            match c {
                ' ' => break,
                '1'..='8' => {
                    file += c as u8 - b'0';
                }
                '/' => {
                    if file != 8 {
                        return Err(());
                    }
                    rank -= 1;
                    file = 0;
                }
                'p' | 'P' | 'n' | 'N' | 'b' | 'B' | 'r' | 'R' | 'q' | 'Q' | 'k' | 'K' => {
                    let color = if c.is_lowercase() {
                        Color::Black
                    } else {
                        Color::White
                    };
                    let piece = Piece::try_from(c)?;
                    if rank > 7 || file > 7 {
                        return Err(());
                    }
                    let square = Square::from_file_rank(file, rank).unwrap();
                    position.set_square(square, piece, color);
                    file += 1;
                }
                _ => {}
            }
        }

        if rank != 0 || file != 8 {
            return Err(());
        }

        let side_to_move = match words.next() {
            Some("w") => Color::White,
            Some("b") => Color::Black,
            _ => return Err(()),
        };
        if side_to_move == Color::Black {
            position.flip_side_to_move();
        }

        let kings = position.pieces_bb(Piece::King);
        let white_king = (kings & position.occupancy_bb(Color::White)).next();
        let black_king = (kings & position.occupancy_bb(Color::Black)).next();

        if white_king.is_none() || black_king.is_none() {
            return Err(());
        }

        let mut rights = CastlingRights::default();
        for c in words.next().ok_or(())?.chars() {
            match c {
                ' ' | '-' => break,
                'K' => {
                    mark_960(&mut position, CastlingSide::Kingside, Color::White);
                    rights = rights.removed_side(Color::White, CastlingSide::Kingside)
                }
                'Q' => {
                    mark_960(&mut position, CastlingSide::Queenside, Color::White);
                    rights = rights.removed_side(Color::White, CastlingSide::Queenside)
                }
                'k' => {
                    mark_960(&mut position, CastlingSide::Kingside, Color::Black);
                    rights = rights.removed_side(Color::Black, CastlingSide::Kingside)
                }
                'q' => {
                    mark_960(&mut position, CastlingSide::Queenside, Color::Black);
                    rights = rights.removed_side(Color::Black, CastlingSide::Queenside)
                }
                c if c.is_alphabetic() => {
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let file = c.to_ascii_lowercase() as u8 - b'a';
                    let king_file = match color {
                        Color::White => white_king.unwrap().file(),
                        Color::Black => black_king.unwrap().file(),
                    };
                    let castling_side = if king_file > file {
                        CastlingSide::Queenside
                    } else {
                        CastlingSide::Kingside
                    };
                    mark_960(&mut position, castling_side, color);
                    rights = rights.removed_side(color, castling_side);
                }
                _ => return Err(()),
            }
        }
        position.set_castling_rights(rights.reversed());

        let ep_square_fen = words.next().unwrap_or("-");
        let ep_square = match ep_square_fen {
            "-" => None,
            _ => Some(Square::from_str(ep_square_fen).map_err(|_| ())?),
        };
        if let Some(ep_square) = ep_square {
            position.set_ep_square(ep_square);
        }

        let halfmove_clock: u8 = words.next().unwrap_or("0").parse().map_err(|_| ())?;
        position.set_halfmove_clock(halfmove_clock);

        let fullmove_number: u16 = words.next().unwrap_or("1").parse().map_err(|_| ())?;
        position.set_fullmove_number(fullmove_number);

        Ok(position)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use crate::core::{
        Position,
        castling_rights::CastlingRights,
        color::Color,
        fen::{Fen, START_POSITION},
        piece::Piece,
        square::Square,
    };

    #[test]
    fn start_position() {
        let mut position = Position::default();
        position.set_square(Square::A1, Piece::Rook, Color::White);
        position.set_square(Square::B1, Piece::Knight, Color::White);
        position.set_square(Square::C1, Piece::Bishop, Color::White);
        position.set_square(Square::D1, Piece::Queen, Color::White);
        position.set_square(Square::E1, Piece::King, Color::White);
        position.set_square(Square::F1, Piece::Bishop, Color::White);
        position.set_square(Square::G1, Piece::Knight, Color::White);
        position.set_square(Square::H1, Piece::Rook, Color::White);
        position.set_square(Square::A2, Piece::Pawn, Color::White);
        position.set_square(Square::B2, Piece::Pawn, Color::White);
        position.set_square(Square::C2, Piece::Pawn, Color::White);
        position.set_square(Square::D2, Piece::Pawn, Color::White);
        position.set_square(Square::E2, Piece::Pawn, Color::White);
        position.set_square(Square::F2, Piece::Pawn, Color::White);
        position.set_square(Square::G2, Piece::Pawn, Color::White);
        position.set_square(Square::H2, Piece::Pawn, Color::White);
        position.set_square(Square::A8, Piece::Rook, Color::Black);
        position.set_square(Square::B8, Piece::Knight, Color::Black);
        position.set_square(Square::C8, Piece::Bishop, Color::Black);
        position.set_square(Square::D8, Piece::Queen, Color::Black);
        position.set_square(Square::E8, Piece::King, Color::Black);
        position.set_square(Square::F8, Piece::Bishop, Color::Black);
        position.set_square(Square::G8, Piece::Knight, Color::Black);
        position.set_square(Square::H8, Piece::Rook, Color::Black);
        position.set_square(Square::A7, Piece::Pawn, Color::Black);
        position.set_square(Square::B7, Piece::Pawn, Color::Black);
        position.set_square(Square::C7, Piece::Pawn, Color::Black);
        position.set_square(Square::D7, Piece::Pawn, Color::Black);
        position.set_square(Square::E7, Piece::Pawn, Color::Black);
        position.set_square(Square::F7, Piece::Pawn, Color::Black);
        position.set_square(Square::G7, Piece::Pawn, Color::Black);
        position.set_square(Square::H7, Piece::Pawn, Color::Black);

        assert_eq!(Fen::from(&position), Fen::from_str(START_POSITION).unwrap());
        assert_eq!(
            Position::try_from(Fen::from_str(START_POSITION).unwrap()).unwrap(),
            position
        );
    }

    #[rstest]
    #[case("2r2rk1/3nqpp1/Q3p1p1/8/2pPR3/2P4P/PP3PP1/2R3K1 b - - 0 20")]
    #[case("r3k2r/p2qppb1/n5p1/1BRPPn1p/5Pb1/2N2N2/1P4PP/2BQK2R b Kkq - 2 14")]
    #[case("r4rk1/pppqp2p/1b1p2p1/3Nn3/2P1p3/3P2Q1/PP2N2P/R1B2R1K w - - 0 20")]
    #[case("r3r1k1/1pp2pp1/p4nbp/3qN3/3P2P1/1PP4P/1P1NQ3/R4RK1 w - - 1 19")]
    #[case("rnbqkbnr/pp4pp/8/3pPp2/8/5N2/PPP2PPP/RNBQKB1R w KQkq f6 0 6")]
    fn reflection(#[case] fen: Fen) {
        let position = Position::try_from(fen.clone()).unwrap();
        assert_eq!(Fen::from(&position), fen);
    }

    #[rstest]
    #[case("2r2rk1/3nqpp1/Q3pp1/8/2pPR3/2P4P/PP3PP1/2R3K1 b - - 0 20")]
    #[case("r3k2r/p2qppb1/n5p1/1BRPPn1p/4Pb1/2N2N2/1P4PP/2BQK2R b - 2 14")]
    #[case("r4rk1/pppqp2p/1b1p2p1/3Nn3/2P1p2/3P2Q1/PP2N2P/R1B2R1K w - - 0 20")]
    #[case("r3r1k1/1pp2pp1/p4nbp/3qN3/3P2P1/1PP4P/1P1NQ3/R4RK1 - - 1 19")]
    #[case("rnbqkbnr/pp4pp/8/3pPp2/8/5N2/PPP2PPP/RNBQKB1R w KQkq 0 6")]
    fn invalid(#[case] fen: Fen) {
        assert_eq!(Position::try_from(fen.clone()), Err(()));
    }

    #[test]
    fn xfen() {
        let position1 = Position::try_from(
            Fen::from_str("rn2k1r1/ppp1pp1p/3p2p1/5bn1/P7/2N2B2/1PPPPP2/2BNK1RR w Gkq - 4 11")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            position1.castling_rights(),
            CastlingRights::new(true, false, true, true)
        );
        assert!(position1.is_chess_960());

        let position2 = Position::try_from(
            Fen::from_str("b1qbrknr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/BNQBRKR1 b Ekq - 3 3")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            position2.castling_rights(),
            CastlingRights::new(false, true, true, true)
        );
        assert!(position2.is_chess_960());
    }
}
