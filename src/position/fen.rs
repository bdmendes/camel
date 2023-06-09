use std::str::FromStr;

use super::{
    board::{Bitboard, Board, Piece},
    CastlingRights, Color, Position, Square,
};

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn board_from_fen(board_fen: &str) -> Result<Board, ()> {
    let mut chars = board_fen.chars();

    let mut rank = 7;
    let mut file = 0;

    let mut pieces: [Bitboard; 6] = [0; 6];
    let mut occupancy: [Bitboard; 2] = [0; 2];

    while let Some(c) = chars.next() {
        match c {
            '1'..='8' => {
                file += (c as u8) - ('0' as u8);
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
                let square = rank * 8 + file;
                pieces[piece as usize] |= 1 << square;
                occupancy[color as usize] |= 1 << square;
                file += 1;
            }
            _ => {}
        }
    }

    if rank == 0 && file == 8 {
        Ok(Board { pieces, occupancy })
    } else {
        Err(())
    }
}

pub fn position_from_fen(fen: &str) -> Result<Position, ()> {
    let mut fen_iter = fen.split_whitespace();

    let board_fen = fen_iter.next().ok_or(())?;
    let board = board_from_fen(board_fen)?;

    let side_to_move = match fen_iter.next() {
        Some("w") => Color::White,
        Some("b") => Color::Black,
        _ => return Err(()),
    };

    let mut castling_rights_fen = fen_iter.next().ok_or(())?.chars();
    let mut castling_rights = CastlingRights::empty();
    while let Some(c) = castling_rights_fen.next() {
        match c {
            ' ' => break,
            'K' => castling_rights |= CastlingRights::WHITE_KINGSIDE,
            'Q' => castling_rights |= CastlingRights::WHITE_QUEENSIDE,
            'k' => castling_rights |= CastlingRights::BLACK_KINGSIDE,
            'q' => castling_rights |= CastlingRights::BLACK_QUEENSIDE,
            '-' => break,
            _ => return Err(()),
        }
    }

    let en_passant_square_fen = fen_iter.next().ok_or(())?;
    let en_passant_square = match en_passant_square_fen {
        "-" => None,
        _ => Square::from_str(en_passant_square_fen).ok(),
    };

    let halfmove_clock: u8 = fen_iter.next().unwrap_or("0").parse().ok().ok_or(())?;

    let fullmove_number: u16 = fen_iter.next().unwrap_or("1").parse().ok().ok_or(())?;

    Ok(Position {
        board,
        side_to_move,
        castling_rights,
        en_passant_square,
        halfmove_clock,
        fullmove_number,
    })
}

pub fn position_to_fen(position: &Position) -> String {
    return String::new();
}
