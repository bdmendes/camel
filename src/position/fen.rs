use super::*;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn position_from_fen(fen: &str) -> Result<Position, String> {
    let mut position = Position {
        board: [Option::None; BOARD_SIZE as usize],
        to_move: Color::White,
        castling_rights: CastlingRights::empty(),
        en_passant_square: None,
        half_move_number: 0,
        full_move_number: 0,
    };

    let mut fen_iter = fen.split_whitespace();
    let board = fen_iter.next().unwrap_or("");
    let next_to_move = fen_iter.next().unwrap_or("w");
    let castling_rights = fen_iter.next().unwrap_or("-");
    let en_passant_square = fen_iter.next().unwrap_or("-");
    let half_move_number = fen_iter.next().unwrap_or("0");
    let full_move_number = fen_iter.next().unwrap_or("1");

    let mut row: u8 = 7;
    let mut col: u8 = 0;
    for c in board.chars() {
        match c {
            '/' => {
                row -= 1;
                col = 0;
            }
            '1'..='8' => {
                col += c as u8 - ('0' as u8);
            }
            _ => {
                let piece = match Piece::from_char(c) {
                    Ok(piece) => piece,
                    Err(msg) => return Err(msg),
                };
                position.board[Square::from_row_col(row, col).index as usize] = Some(piece);
                col += 1;
            }
        }
    }

    position.to_move = match next_to_move {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return Err("Invalid next to move".to_owned()),
    };

    for c in castling_rights.chars() {
        match c {
            'K' => position.castling_rights |= CastlingRights::WHITE_KINGSIDE,
            'Q' => position.castling_rights |= CastlingRights::WHITE_QUEENSIDE,
            'k' => position.castling_rights |= CastlingRights::BLACK_KINGSIDE,
            'q' => position.castling_rights |= CastlingRights::BLACK_QUEENSIDE,
            '-' => break,
            _ => panic!("Invalid castling rights"),
        }
    }

    position.en_passant_square = match en_passant_square {
        "-" => None,
        _ => Some(Square::from_algebraic(en_passant_square)),
    };

    position.half_move_number = half_move_number.parse().unwrap_or(0);
    position.full_move_number = full_move_number.parse().unwrap_or(1);

    Ok(position)
}

pub fn position_to_fen(position: &Position, omit_move_numbers: bool) -> String {
    let mut fen = String::new();

    for row in (0..8).rev() {
        let mut empty = 0;
        for col in 0..8 {
            match position.at(&Square::from_row_col(row, col)) {
                None => empty += 1,
                Some(piece) => {
                    if empty > 0 {
                        fen.push((empty + ('0' as u8)) as char);
                        empty = 0;
                    }
                    fen.push(piece.to_char());
                }
            }
        }
        if empty > 0 {
            fen.push((empty + ('0' as u8)) as char);
        }
        if row > 0 {
            fen.push('/');
        }
    }

    fen.push(' ');
    fen.push(match position.to_move {
        Color::White => 'w',
        Color::Black => 'b',
    });

    fen.push(' ');
    let mut castling_rights = String::new();
    if position
        .castling_rights
        .contains(CastlingRights::WHITE_KINGSIDE)
    {
        castling_rights.push('K');
    }
    if position
        .castling_rights
        .contains(CastlingRights::WHITE_QUEENSIDE)
    {
        castling_rights.push('Q');
    }
    if position
        .castling_rights
        .contains(CastlingRights::BLACK_KINGSIDE)
    {
        castling_rights.push('k');
    }
    if position
        .castling_rights
        .contains(CastlingRights::BLACK_QUEENSIDE)
    {
        castling_rights.push('q');
    }
    if castling_rights.is_empty() {
        castling_rights.push('-');
    }
    fen.push_str(castling_rights.as_str());

    fen.push(' ');
    match position.en_passant_square {
        None => fen.push('-'),
        Some(square) => fen.push_str(&square.to_algebraic()),
    }

    if !omit_move_numbers {
        fen.push(' ');
        fen.push_str(position.half_move_number.to_string().as_str());
        fen.push(' ');
        fen.push_str(&position.full_move_number.to_string());
    }

    fen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_start() {
        let position = position_from_fen(START_FEN).unwrap();
        assert_eq!(position_to_fen(&position, false), START_FEN);

        let expected_board_view = vec![
            "rnbqkbnr", //
            "pppppppp", //
            "        ", //
            "        ", //
            "        ", //
            "        ", //
            "PPPPPPPP", //
            "RNBQKBNR",
        ]
        .join("\n");
        assert_eq!(format!("{}", position), expected_board_view);
    }

    #[test]
    fn fen_midgame() {
        let midgame_fen = "r4rk1/pp1q1ppp/2n2b2/3Q4/8/2N5/PPP2PPP/R3K1NR b KQ - 0 14";
        let position = position_from_fen(midgame_fen).unwrap();
        assert_eq!(position_to_fen(&position, false), midgame_fen);

        let expected_board_view = vec![
            "r    rk ", //
            "pp q ppp", //
            "  n  b  ", //
            "   Q    ", //
            "        ", //
            "  N     ", //
            "PPP  PPP", //
            "R   K NR",
        ]
        .join("\n");
        assert_eq!(format!("{}", position), expected_board_view);
    }
}
