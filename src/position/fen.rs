use super::{
    board::{Board, Piece},
    CastlingRights, Color, Position, Square,
};
use std::str::FromStr;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const KIWIPETE_WHITE_FEN: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const KIWIPETE_BLACK_FEN: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1";

fn chess960_compliant(castling_rights: CastlingRights, board: Board) -> bool {
    let white_can_castle_kingside = castling_rights.contains(CastlingRights::WHITE_KINGSIDE);
    let white_can_castle_queenside = castling_rights.contains(CastlingRights::WHITE_QUEENSIDE);

    let black_can_castle_kingside = castling_rights.contains(CastlingRights::BLACK_KINGSIDE);
    let black_can_castle_queenside = castling_rights.contains(CastlingRights::BLACK_QUEENSIDE);

    let white_can_castle = white_can_castle_kingside || white_can_castle_queenside;
    let black_can_castle = black_can_castle_kingside || black_can_castle_queenside;

    if (white_can_castle && board.piece_color_at(Square::E1) != Some((Piece::King, Color::White)))
        || (black_can_castle
            && board.piece_color_at(Square::E8) != Some((Piece::King, Color::Black)))
    {
        return true;
    }

    if (white_can_castle_kingside
        && board.piece_color_at(Square::H1) != Some((Piece::Rook, Color::White)))
        || (black_can_castle_kingside
            && board.piece_color_at(Square::H8) != Some((Piece::Rook, Color::Black)))
    {
        return true;
    }

    if (white_can_castle_queenside
        && board.piece_color_at(Square::A1) != Some((Piece::Rook, Color::White)))
        || (black_can_castle_queenside
            && board.piece_color_at(Square::A8) != Some((Piece::Rook, Color::Black)))
    {
        return true;
    }

    false
}

pub fn board_from_fen(board_fen: &str) -> Option<Board> {
    let chars = board_fen.chars();

    let mut rank = 7;
    let mut file = 0;

    let mut board = Board::new();

    for c in chars {
        match c {
            ' ' => break,
            '1'..='8' => {
                file += (c as u8) - b'0';
            }
            '/' => {
                rank -= 1;
                file = 0;
            }
            'p' | 'P' | 'n' | 'N' | 'b' | 'B' | 'r' | 'R' | 'q' | 'Q' | 'k' | 'K' => {
                let color = if c.is_lowercase() { Color::Black } else { Color::White };
                let piece = match c.to_ascii_lowercase() {
                    'p' => Piece::Pawn,
                    'n' => Piece::Knight,
                    'b' => Piece::Bishop,
                    'r' => Piece::Rook,
                    'q' => Piece::Queen,
                    'k' => Piece::King,
                    _ => unreachable!(),
                };
                if rank > 7 || file > 7 {
                    return None;
                }
                let square = Square::from(rank * 8 + file).unwrap();
                board.set_square::<true>(square, piece, color);
                file += 1;
            }
            _ => {}
        }
    }

    if rank == 0 && file == 8 {
        Some(board)
    } else {
        None
    }
}

pub fn position_from_fen(fen: &str) -> Option<Position> {
    let mut fen_iter = fen.split_whitespace();

    let board_fen = fen_iter.next()?;
    let board = board_from_fen(board_fen)?;

    let side_to_move = match fen_iter.next() {
        Some("w") => Color::White,
        Some("b") => Color::Black,
        _ => return None,
    };

    let mut is_chess960 = false;
    let castling_rights_fen = fen_iter.next()?.chars();
    let mut castling_rights = CastlingRights::empty();
    for c in castling_rights_fen {
        match c {
            ' ' => break,
            'K' => castling_rights |= CastlingRights::WHITE_KINGSIDE,
            'Q' => castling_rights |= CastlingRights::WHITE_QUEENSIDE,
            'k' => castling_rights |= CastlingRights::BLACK_KINGSIDE,
            'q' => castling_rights |= CastlingRights::BLACK_QUEENSIDE,
            '-' => break,
            _ => {
                // Other letters are used as the file in Chess960.
                is_chess960 = true;

                let color = if c.is_lowercase() { Color::Black } else { Color::White };
                let file = match c.to_ascii_lowercase() {
                    'a' => 0,
                    'b' => 1,
                    'c' => 2,
                    'd' => 3,
                    'e' => 4,
                    'f' => 5,
                    'g' => 6,
                    'h' => 7,
                    _ => return None,
                };
                let color_king_square = board.pieces_bb_color(Piece::King, color);
                if let Some(color_king_square) = color_king_square.into_iter().next() {
                    let king_file = color_king_square.file();
                    if file > king_file {
                        castling_rights |= match color {
                            Color::White => CastlingRights::WHITE_KINGSIDE,
                            Color::Black => CastlingRights::BLACK_KINGSIDE,
                        };
                    } else {
                        castling_rights |= match color {
                            Color::White => CastlingRights::WHITE_QUEENSIDE,
                            Color::Black => CastlingRights::BLACK_QUEENSIDE,
                        };
                    }
                } else {
                    return None;
                }
            }
        }
    }

    if !is_chess960 && chess960_compliant(castling_rights, board) {
        is_chess960 = true;
    }

    let en_passant_square_fen = fen_iter.next()?;
    let en_passant_square = match en_passant_square_fen {
        "-" => None,
        _ => Square::from_str(en_passant_square_fen).ok(),
    };

    let halfmove_clock: u8 = fen_iter.next().unwrap_or("0").parse().ok()?;

    let fullmove_number: u16 = fen_iter.next().unwrap_or("1").parse().ok()?;

    Some(Position {
        board,
        side_to_move,
        castling_rights,
        en_passant_square,
        halfmove_clock,
        fullmove_number,
        is_chess960,
    })
}

fn board_to_fen(board: &Board) -> String {
    let mut fen = String::new();

    for rank in (0..8).rev() {
        let mut empty_squares = 0;

        if rank != 7 {
            fen.push('/');
        }

        for file in 0..8 {
            let square = rank * 8 + file;

            let piece = match board.piece_color_at(Square::from(square).unwrap()) {
                Some((Piece::Pawn, Color::White)) => 'P',
                Some((Piece::Pawn, Color::Black)) => 'p',
                Some((Piece::Knight, Color::White)) => 'N',
                Some((Piece::Knight, Color::Black)) => 'n',
                Some((Piece::Bishop, Color::White)) => 'B',
                Some((Piece::Bishop, Color::Black)) => 'b',
                Some((Piece::Rook, Color::White)) => 'R',
                Some((Piece::Rook, Color::Black)) => 'r',
                Some((Piece::Queen, Color::White)) => 'Q',
                Some((Piece::Queen, Color::Black)) => 'q',
                Some((Piece::King, Color::White)) => 'K',
                Some((Piece::King, Color::Black)) => 'k',
                None => ' ',
            };

            if piece == ' ' {
                empty_squares += 1;
            } else {
                if empty_squares > 0 {
                    fen.push_str(&empty_squares.to_string());
                    empty_squares = 0;
                }
                fen.push(piece);
            }
        }

        if empty_squares > 0 {
            fen.push_str(&empty_squares.to_string());
        }
    }

    fen
}

pub fn position_to_fen(position: &Position) -> String {
    let mut fen = String::new();

    fen.push_str(&board_to_fen(&position.board));

    fen.push(' ');

    fen.push_str(match position.side_to_move {
        Color::White => "w",
        Color::Black => "b",
    });

    fen.push(' ');

    if position.castling_rights.is_empty() {
        fen.push('-');
    } else {
        if position.castling_rights.contains(CastlingRights::WHITE_KINGSIDE) {
            fen.push('K');
        }
        if position.castling_rights.contains(CastlingRights::WHITE_QUEENSIDE) {
            fen.push('Q');
        }
        if position.castling_rights.contains(CastlingRights::BLACK_KINGSIDE) {
            fen.push('k');
        }
        if position.castling_rights.contains(CastlingRights::BLACK_QUEENSIDE) {
            fen.push('q');
        }
    }

    fen.push(' ');

    fen.push_str(
        &position.en_passant_square.map(|sq| sq.to_string()).unwrap_or_else(|| "-".to_string()),
    );

    fen.push(' ');

    fen.push_str(&position.halfmove_clock.to_string());

    fen.push(' ');

    fen.push_str(&position.fullmove_number.to_string());

    fen
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::fen::Piece::*;
    use crate::position::Color::*;

    #[test]
    fn fails_when_fen_is_invalid() {
        let invalid_fens = [
            "pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // missing a rank
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR - KQkq - 0 1", // missing side to move
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",   // missing castling rights
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq", // missing en passant square
        ];

        for fen in &invalid_fens {
            assert!(position_from_fen(fen).is_none());
        }
    }

    #[test]
    fn parses_board() {
        let position = position_from_fen(KIWIPETE_WHITE_FEN).unwrap();

        let pieces_map = [
            Some((Rook, White)),   // a1
            None,                  // b1
            None,                  // c1
            None,                  // d1
            Some((King, White)),   // e1
            None,                  // f1
            None,                  // g1
            Some((Rook, White)),   // h1
            Some((Pawn, White)),   // a2
            Some((Pawn, White)),   // b2
            Some((Pawn, White)),   // c2
            Some((Bishop, White)), // d2
            Some((Bishop, White)), // e2
            Some((Pawn, White)),   // f2
            Some((Pawn, White)),   // g2
            Some((Pawn, White)),   // h2
            None,                  // a3
            None,                  // b3
            Some((Knight, White)), // c3
            None,                  // d3
            None,                  // e3
            Some((Queen, White)),  // f3
            None,                  // g3
            Some((Pawn, Black)),   // h3
            None,                  // a4
            Some((Pawn, Black)),   // b4
            None,                  // c4
            None,                  // d4
            Some((Pawn, White)),   // e4
            None,                  // f4
            None,                  // g4
            None,                  // h4
            None,                  // a5
            None,                  // b5
            None,                  // c5
            Some((Pawn, White)),   // d5
            Some((Knight, White)), // e5
            None,                  // f5
            None,                  // g5
            None,                  // h5
            Some((Bishop, Black)), // a6
            Some((Knight, Black)), // b6
            None,                  // c6
            None,                  // d6
            Some((Pawn, Black)),   // e6
            Some((Knight, Black)), // f6
            Some((Pawn, Black)),   // g6
            None,                  // h6
            Some((Pawn, Black)),   // a7
            None,                  // b7
            Some((Pawn, Black)),   // c7
            Some((Pawn, Black)),   // d7
            Some((Queen, Black)),  // e7
            Some((Pawn, Black)),   // f7
            Some((Bishop, Black)), // g7
            None,                  // h7
            Some((Rook, Black)),   // a8
            None,                  // b8
            None,                  // c8
            None,                  // d8
            Some((King, Black)),   // e8
            None,                  // f8
            None,                  // g8
            Some((Rook, Black)),   // h8
        ];

        for (i, piece) in pieces_map.iter().enumerate() {
            let square = Square::from(i as u8).unwrap();
            assert_eq!(position.board.piece_color_at(square), *piece);
        }
    }

    #[test]
    fn parses_position_info() {
        let position = position_from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert_eq!(position.side_to_move, White);
        assert_eq!(
            position.castling_rights,
            CastlingRights::WHITE_KINGSIDE
                | CastlingRights::WHITE_QUEENSIDE
                | CastlingRights::BLACK_KINGSIDE
                | CastlingRights::BLACK_QUEENSIDE
        );
        assert_eq!(position.en_passant_square, None);
        assert_eq!(position.halfmove_clock, 0);
        assert_eq!(position.fullmove_number, 1);
    }

    #[test]
    fn to_fen_reflexive() {
        let position = position_from_fen(KIWIPETE_WHITE_FEN).unwrap();
        assert_eq!(position_to_fen(&position), KIWIPETE_WHITE_FEN);
    }
}
