use std::{fmt::Display, str::FromStr};

use super::{
    castling_rights::{CastlingRights, CastlingSide},
    color::Color,
    piece::Piece,
    square::Square,
    Position,
};

pub const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Fen(String);

impl Fen {
    pub fn new(str: &str) -> Self {
        Self(str.to_string())
    }
}

impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<Position> for Fen {
    fn from(position: Position) -> Self {
        let mut str = String::new();
        let mut empty_spaces = 0;

        for rank in (0..=7).rev() {
            for file in 0..=7 {
                let square = Square::from(rank * 8 + file).unwrap();
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
        let mut position = Position::default();
        let mut words = fen.0.split_whitespace();
        let mut rank: u64 = 7;
        let mut file: u64 = 0;

        for c in words.next().ok_or(())?.chars() {
            match c {
                ' ' => break,
                '1'..='8' => {
                    file += (c as u64) - (b'0') as u64;
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
                    let square = Square::from(rank * 8 + file).unwrap();
                    position.set_square(square, piece, color);
                    file += 1;
                }
                _ => {}
            }
        }

        if rank != 0 {
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

        let mut rights = CastlingRights::default();
        for c in words.next().ok_or(())?.chars() {
            match c {
                ' ' => break,
                'K' => rights = rights.removed_side(Color::White, CastlingSide::Kingside),
                'Q' => rights = rights.removed_side(Color::White, CastlingSide::Queenside),
                'k' => rights = rights.removed_side(Color::Black, CastlingSide::Kingside),
                'q' => rights = rights.removed_side(Color::Black, CastlingSide::Queenside),
                '-' => break,
                _ => {
                    todo!("chess 960 port.");
                }
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
    use crate::position::{
        color::Color,
        fen::{Fen, START_POSITION},
        piece::Piece,
        square::Square,
        Position,
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

        assert_eq!(Fen::from(position.clone()), Fen::new(START_POSITION));
        assert_eq!(
            Position::try_from(Fen::new(START_POSITION)).unwrap(),
            position
        );
    }

    #[test]
    fn reflection() {
        let fens = [
            "2r2rk1/3nqpp1/Q3p1p1/8/2pPR3/2P4P/PP3PP1/2R3K1 b - - 0 20",
            "r3k2r/p2qppb1/n5p1/1BRPPn1p/5Pb1/2N2N2/1P4PP/2BQK2R b Kkq - 2 14",
            "r4rk1/pppqp2p/1b1p2p1/3Nn3/2P1p3/3P2Q1/PP2N2P/R1B2R1K w - - 0 20",
            "r3r1k1/1pp2pp1/p4nbp/3qN3/3P2P1/1PP4P/1P1NQ3/R4RK1 w - - 1 19",
            "rnbqkbnr/pp4pp/8/3pPp2/8/5N2/PPP2PPP/RNBQKB1R w KQkq f6 0 6",
        ];
        for fen in fens {
            let fen = Fen::new(fen);
            assert_eq!(Fen::from(Position::try_from(fen.clone()).unwrap()), fen);
        }
    }

    #[test]
    fn invalid() {
        let fens = [
            "2r2rk1/3nqpp1/Q3pp1/8/2pPR3/2P4P/PP3PP1/2R3K1 b - - 0 20",
            "r3k2r/p2qppb1/n5p1/1BRPPn1p/4Pb1/2N2N2/1P4PP/2BQK2R b - 2 14",
            "r4rk1/pppqp2p/1b1p2p1/3Nn3/2P1p2/3P2Q1/PP2N2P/R1B2R1K w - - 0 20",
            "r3r1k1/1pp2pp1/p4nbp/3qN3/3P2P1/1PP4P/1P1NQ3/R4RK1 - - 1 19",
            "rnbqkbnr/pp4pp/8/3pPp2/8/5N2/PPP2PPP/RNBQKB1R w KQkq 0 6",
        ];
        for fen in fens {
            let fen = Fen::new(fen);
            assert_eq!(Position::try_from(fen.clone()), Err(()));
        }
    }
}
