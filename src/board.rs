pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Square {
    pub row: usize,
    pub col: usize,
}

pub struct Position {
    pub board: [[Option<(Piece, Color)>; 8]; 8],
    pub next_to_move: Color,
    pub white_can_castle_kingside: bool,
    pub white_can_castle_queenside: bool,
    pub black_can_castle_kingside: bool,
    pub black_can_castle_queenside: bool,
    pub en_passant_square: Option<Square>,
    pub half_move_number: u8,
    pub full_move_number: u16,
}

impl Color {
    pub fn opposing_color(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Square {
    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push((('a' as u8) + self.col as u8) as char);
        algebraic.push((('8' as u8) - self.row as u8) as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: &str) -> Square {
        let mut chars = algebraic.chars();
        let col = chars.next().unwrap() as u8 - ('a' as u8);
        let row = ('8' as u8) - chars.next().unwrap() as u8;
        Square {
            row: row as usize,
            col: col as usize,
        }
    }
}

impl Piece {
    pub fn from_char(c: char) -> Piece {
        match c {
            'p' => Piece::Pawn,
            'r' => Piece::Rook,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => panic!("Invalid piece"),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Rook => 'r',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }

    pub fn to_char_with_color(&self, color: Color) -> char {
        let c = self.to_char();
        match color {
            Color::White => c.to_uppercase().next().unwrap(),
            Color::Black => c,
        }
    }
}

impl Position {
    pub fn from_fen(fen: &str) -> Position {
        let mut position = Position {
            board: [[Option::None; 8]; 8],
            next_to_move: Color::White,
            white_can_castle_kingside: false,
            white_can_castle_queenside: false,
            black_can_castle_kingside: false,
            black_can_castle_queenside: false,
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
                'K' => position.white_can_castle_kingside = true,
                'Q' => position.white_can_castle_queenside = true,
                'k' => position.black_can_castle_kingside = true,
                'q' => position.black_can_castle_queenside = true,
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

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for row in 0..8 {
            let mut empty = 0;
            for col in 0..8 {
                match self.board[row][col] {
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
        let mut castling_rights = String::new();
        if self.white_can_castle_kingside {
            castling_rights.push('K');
        }
        if self.white_can_castle_queenside {
            castling_rights.push('Q');
        }
        if self.black_can_castle_kingside {
            castling_rights.push('k');
        }
        if self.black_can_castle_queenside {
            castling_rights.push('q');
        }
        if castling_rights.is_empty() {
            castling_rights.push('-');
        }
        fen.push_str(castling_rights.as_str());

        fen.push(' ');
        match self.en_passant_square {
            None => fen.push('-'),
            Some(square) => fen.push_str(&square.to_algebraic()),
        }

        fen.push(' ');
        fen.push_str(self.half_move_number.to_string().as_str());

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
                    Some((piece, color)) => {
                        string.push(piece.to_char_with_color(color));
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
