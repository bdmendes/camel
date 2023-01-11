pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone)]
pub enum Piece {
    Pawn(Color),
    Rook(Color),
    Knight(Color),
    Bishop(Color),
    Queen(Color),
    King(Color),
}

#[derive(Copy, Clone)]
pub struct Square {
    pub row: u8,
    pub col: u8,
}

pub enum CastlingRights {
    WhiteKingSide,
    WhiteQueenSide,
    BlackKingSide,
    BlackQueenSide,
}

pub struct Position {
    pub board: [[Option<Piece>; 8]; 8],
    pub next_to_move: Color,
    pub castling_rights: Vec<CastlingRights>,
    pub en_passant_square: Option<Square>,
    pub half_move_clock: u8,
    pub full_move_number: u16,
}

impl Position {
    pub fn from_fen(fen: &str) -> Position {
        let mut position = Position {
            board: [[Option::None; 8]; 8],
            next_to_move: Color::White,
            castling_rights: Vec::with_capacity(4),
            en_passant_square: None,
            half_move_clock: 0,
            full_move_number: 0,
        };

        let mut fen_iter = fen.split_whitespace();
        let board = fen_iter.next().unwrap();
        let next_to_move = fen_iter.next().unwrap();
        let castling_rights = fen_iter.next().unwrap();
        let en_passant_square = fen_iter.next().unwrap();
        let half_move_clock = fen_iter.next().unwrap();
        let full_move_number = fen_iter.next().unwrap();

        let mut row: usize = 0;
        let mut col: usize = 0;
        for c in board.chars() {
            match c {
                '/' => {
                    row += 1;
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
                    position.board[row][col] = match c.to_lowercase().next().unwrap() {
                        'p' => Some(Piece::Pawn(color)),
                        'r' => Some(Piece::Rook(color)),
                        'n' => Some(Piece::Knight(color)),
                        'b' => Some(Piece::Bishop(color)),
                        'q' => Some(Piece::Queen(color)),
                        'k' => Some(Piece::King(color)),
                        _ => panic!("Invalid piece"),
                    };
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
                'K' => position.castling_rights.push(CastlingRights::WhiteKingSide),
                'Q' => position
                    .castling_rights
                    .push(CastlingRights::WhiteQueenSide),
                'k' => position.castling_rights.push(CastlingRights::BlackKingSide),
                'q' => position
                    .castling_rights
                    .push(CastlingRights::BlackQueenSide),
                '-' => break,
                _ => panic!("Invalid castling rights"),
            }
        }

        position.en_passant_square = match en_passant_square {
            "-" => None,
            _ => Some(Square::from_algebraic(en_passant_square)),
        };

        position.half_move_clock = half_move_clock.parse().unwrap();

        position.full_move_number = full_move_number.parse().unwrap();

        position
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for row in 0..8 {
            let mut empty = 0;
            for col in 0..8 {
                match self.board[row][col] {
                    None => empty += 1,
                    Some(piece) => {
                        if empty > 0 {
                            fen.push((empty + ('0' as u8)) as char);
                            empty = 0;
                        }
                        let (piece_char, color) = match piece {
                            Piece::Pawn(color) => ('p', color),
                            Piece::Rook(color) => ('r', color),
                            Piece::Knight(color) => ('n', color),
                            Piece::Bishop(color) => ('b', color),
                            Piece::Queen(color) => ('q', color),
                            Piece::King(color) => ('k', color),
                        };
                        fen.push(match color {
                            Color::White => piece_char.to_uppercase().next().unwrap(),
                            Color::Black => piece_char,
                        });
                    }
                }
            }
            if empty > 0 {
                fen.push((empty + ('0' as u8)) as char);
            }
            if row < 7 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(match self.next_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        fen.push(' ');
        for castling_right in self.castling_rights.iter() {
            fen.push(match castling_right {
                CastlingRights::WhiteKingSide => 'K',
                CastlingRights::WhiteQueenSide => 'Q',
                CastlingRights::BlackKingSide => 'k',
                CastlingRights::BlackQueenSide => 'q',
            });
        }

        fen.push(' ');
        match self.en_passant_square {
            None => fen.push('-'),
            Some(square) => fen.push_str(&square.to_algebraic()),
        }

        fen.push(' ');
        fen.push_str(self.half_move_clock.to_string().as_str());

        fen.push(' ');
        fen.push_str(&self.full_move_number.to_string());

        fen
    }

    pub fn to_string(&self) -> String {
        let mut string = String::new();

        for row in 0..8 {
            for col in 0..8 {
                match self.board[row][col] {
                    None => string.push(' '),
                    Some(piece) => {
                        let (piece_char, color) = match piece {
                            Piece::Pawn(color) => ('p', color),
                            Piece::Rook(color) => ('r', color),
                            Piece::Knight(color) => ('n', color),
                            Piece::Bishop(color) => ('b', color),
                            Piece::Queen(color) => ('q', color),
                            Piece::King(color) => ('k', color),
                        };
                        let colored_piece_char = match color {
                            Color::White => piece_char.to_uppercase().next().unwrap(),
                            Color::Black => piece_char,
                        };
                        string.push(colored_piece_char);
                    }
                }
            }
            if row < 7 {
                string.push('\n');
            }
        }

        string
    }
}

impl Square {
    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push((('a' as u8) + self.col) as char);
        algebraic.push((('8' as u8) - self.row) as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: &str) -> Square {
        let mut chars = algebraic.chars();
        let col = chars.next().unwrap() as u8 - ('a' as u8);
        let row = ('8' as u8) - chars.next().unwrap() as u8;
        Square { row, col }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fen_start() {
        let position = Position::from_fen(START_FEN);
        assert_eq!(position.to_fen(), START_FEN);

        let board_view = position.to_string();
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
        assert_eq!(board_view, expected_board_view);
    }

    #[test]
    fn fen_midgame() {
        let midgame_fen = "r4rk1/pp1q1ppp/2n2b2/3Q4/8/2N5/PPP2PPP/R3K1NR b KQ - 0 14";
        let position = Position::from_fen(midgame_fen);
        assert_eq!(position.to_fen(), midgame_fen);

        let board_view = position.to_string();
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
        assert_eq!(board_view, expected_board_view);
    }

    #[test]
    fn algebraic_square() {
        let square = Square::from_algebraic("a1");
        assert_eq!(square.to_algebraic(), "a1");
        assert_eq!(square.row, 7);
        assert_eq!(square.col, 0);

        let square = Square::from_algebraic("h8");
        assert_eq!(square.to_algebraic(), "h8");
        assert_eq!(square.row, 0);
        assert_eq!(square.col, 7);

        let square = Square::from_algebraic("e4");
        assert_eq!(square.to_algebraic(), "e4");
        assert_eq!(square.row, 4);
        assert_eq!(square.col, 4);

        let square = Square::from_algebraic("d5");
        assert_eq!(square.to_algebraic(), "d5");
        assert_eq!(square.row, 3);
        assert_eq!(square.col, 3);
    }
}
