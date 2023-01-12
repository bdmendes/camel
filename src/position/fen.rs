use super::*;

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn position_from_fen(fen: &str) -> Position {
    let mut position = Position {
        board: [[Option::None; 8]; 8],
        next_to_move: Color::White,
        castling_rights: CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        },
        en_passant_square: None,
        half_move_number: 0,
        full_move_number: 0,
    };

    let mut fen_iter = fen.split_whitespace();
    let board = fen_iter.next().unwrap();
    let next_to_move = fen_iter.next().unwrap();
    let castling_rights = fen_iter.next().unwrap();
    let en_passant_square = fen_iter.next().unwrap();
    let half_move_number = fen_iter.next().unwrap();
    let full_move_number = fen_iter.next().unwrap();

    let mut row: usize = 7;
    let mut col: usize = 0;
    for c in board.chars() {
        match c {
            '/' => {
                row -= 1;
                col = 0;
            }
            '1'..='8' => {
                col += c as usize - ('0' as usize);
            }
            _ => {
                let color = if c.is_lowercase() {
                    Color::Black
                } else {
                    Color::White
                };
                let piece = Piece::from_char(c.to_lowercase().next().unwrap());
                position.board[row][col] = Some((piece, color));
                col += 1;
            }
        }
    }

    position.next_to_move = match next_to_move {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invalid next to move"),
    };

    for c in castling_rights.chars() {
        match c {
            'K' => position.castling_rights.white_kingside = true,
            'Q' => position.castling_rights.white_queenside = true,
            'k' => position.castling_rights.black_kingside = true,
            'q' => position.castling_rights.black_queenside = true,
            '-' => break,
            _ => panic!("Invalid castling rights"),
        }
    }

    position.en_passant_square = match en_passant_square {
        "-" => None,
        _ => Some(Square::from_algebraic(en_passant_square)),
    };

    position.half_move_number = half_move_number.parse().unwrap();
    position.full_move_number = full_move_number.parse().unwrap();

    position
}

pub fn position_to_fen(position: &Position) -> String {
    let mut fen = String::new();

    for row in (0..8).rev() {
        let mut empty = 0;
        for col in 0..8 {
            match position.board[row][col] {
                None => empty += 1,
                Some((piece, color)) => {
                    if empty > 0 {
                        fen.push((empty + ('0' as u8)) as char);
                        empty = 0;
                    }
                    fen.push(piece.to_char_with_color(color));
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
    fen.push(match position.next_to_move {
        Color::White => 'w',
        Color::Black => 'b',
    });

    fen.push(' ');
    let mut castling_rights = String::new();
    if position.castling_rights.white_kingside {
        castling_rights.push('K');
    }
    if position.castling_rights.white_queenside {
        castling_rights.push('Q');
    }
    if position.castling_rights.black_kingside {
        castling_rights.push('k');
    }
    if position.castling_rights.black_queenside {
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

    fen.push(' ');
    fen.push_str(position.half_move_number.to_string().as_str());

    fen.push(' ');
    fen.push_str(&position.full_move_number.to_string());

    fen
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_start() {
        let position = position_from_fen(START_FEN);
        assert_eq!(position_to_fen(&position), START_FEN);

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
        let position = position_from_fen(midgame_fen);
        assert_eq!(position_to_fen(&position), midgame_fen);

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
