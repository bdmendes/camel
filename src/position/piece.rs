pub const UP: i8 = 8;
pub const DOWN: i8 = -8;
pub const LEFT: i8 = -1;
pub const RIGHT: i8 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposing(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn(Color),
    Rook(Color),
    Knight(Color),
    Bishop(Color),
    Queen(Color),
    King(Color),
}

impl Piece {
    pub fn from_char(c: char) -> Result<Piece, String> {
        let color = match c.is_uppercase() {
            true => Color::White,
            false => Color::Black,
        };
        match c {
            'p' | 'P' => Ok(Piece::Pawn(color)),
            'r' | 'R' => Ok(Piece::Rook(color)),
            'n' | 'N' => Ok(Piece::Knight(color)),
            'b' | 'B' => Ok(Piece::Bishop(color)),
            'q' | 'Q' => Ok(Piece::Queen(color)),
            'k' | 'K' => Ok(Piece::King(color)),
            _ => Err("Invalid piece".to_owned()),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Piece::Pawn(Color::White) => 'P',
            Piece::Rook(Color::White) => 'R',
            Piece::Knight(Color::White) => 'N',
            Piece::Bishop(Color::White) => 'B',
            Piece::Queen(Color::White) => 'Q',
            Piece::King(Color::White) => 'K',
            Piece::Pawn(Color::Black) => 'p',
            Piece::Rook(Color::Black) => 'r',
            Piece::Knight(Color::Black) => 'n',
            Piece::Bishop(Color::Black) => 'b',
            Piece::Queen(Color::Black) => 'q',
            Piece::King(Color::Black) => 'k',
        }
    }

    pub fn unchecked_directions(&self) -> Vec<i8> {
        match self {
            Piece::Pawn(Color::White) => vec![UP],
            Piece::Pawn(Color::Black) => vec![DOWN],
            Piece::Rook(_) => vec![UP, DOWN, LEFT, RIGHT],
            Piece::Knight(_) => vec![
                2 * UP + LEFT,
                2 * UP + RIGHT,
                2 * DOWN + LEFT,
                2 * DOWN + RIGHT,
                2 * LEFT + UP,
                2 * LEFT + DOWN,
                2 * RIGHT + UP,
                2 * RIGHT + DOWN,
            ],
            Piece::Bishop(_) => vec![UP + LEFT, UP + RIGHT, DOWN + LEFT, DOWN + RIGHT],
            Piece::Queen(_) | Piece::King(_) => vec![
                UP,
                DOWN,
                LEFT,
                RIGHT,
                UP + LEFT,
                UP + RIGHT,
                DOWN + LEFT,
                DOWN + RIGHT,
            ],
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Piece::Pawn(color)
            | Piece::Rook(color)
            | Piece::Knight(color)
            | Piece::Bishop(color)
            | Piece::Queen(color)
            | Piece::King(color) => *color,
        }
    }

    pub fn is_crawling(&self) -> bool {
        match self {
            Piece::Pawn(_) | Piece::King(_) | Piece::Knight(_) => false,
            _ => true,
        }
    }
}
