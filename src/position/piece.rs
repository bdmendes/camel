pub const UP: i64 = 8;
pub const DOWN: i64 = -8;
pub const LEFT: i64 = -1;
pub const RIGHT: i64 = 1;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    WP,
    WR,
    WN,
    WB,
    WQ,
    WK,
    BP,
    BR,
    BN,
    BB,
    BQ,
    BK,
}

impl Piece {
    pub fn color(&self) -> Color {
        match self {
            Piece::WP | Piece::WR | Piece::WN | Piece::WB | Piece::WQ | Piece::WK => Color::White,
            Piece::BP | Piece::BR | Piece::BN | Piece::BB | Piece::BQ | Piece::BK => Color::Black,
        }
    }

    pub fn from_char(c: char) -> Result<Piece, String> {
        match c {
            'p' => Ok(Piece::BP),
            'r' => Ok(Piece::BR),
            'n' => Ok(Piece::BN),
            'b' => Ok(Piece::BB),
            'q' => Ok(Piece::BQ),
            'k' => Ok(Piece::BK),
            'P' => Ok(Piece::WP),
            'R' => Ok(Piece::WR),
            'N' => Ok(Piece::WN),
            'B' => Ok(Piece::WB),
            'Q' => Ok(Piece::WQ),
            'K' => Ok(Piece::WK),
            _ => Err(format!("Invalid piece character: {}", c)),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Piece::WP => 'P',
            Piece::WR => 'R',
            Piece::WN => 'N',
            Piece::WB => 'B',
            Piece::WQ => 'Q',
            Piece::WK => 'K',
            Piece::BP => 'p',
            Piece::BR => 'r',
            Piece::BN => 'n',
            Piece::BB => 'b',
            Piece::BQ => 'q',
            Piece::BK => 'k',
        }
    }

    pub fn unchecked_directions(&self) -> &'static [i64] {
        match self {
            Piece::WP => &[UP, UP + UP, UP + LEFT, UP + RIGHT],
            Piece::BP => &[DOWN, DOWN + DOWN, DOWN + LEFT, DOWN + RIGHT],
            Piece::WR | Piece::BR => &[UP, DOWN, LEFT, RIGHT],
            Piece::WN | Piece::BN => &[
                2 * UP + LEFT,
                2 * UP + RIGHT,
                2 * DOWN + LEFT,
                2 * DOWN + RIGHT,
                2 * LEFT + UP,
                2 * LEFT + DOWN,
                2 * RIGHT + UP,
                2 * RIGHT + DOWN,
            ],
            Piece::WB | Piece::BB => &[UP + LEFT, UP + RIGHT, DOWN + LEFT, DOWN + RIGHT],
            Piece::WQ | Piece::BQ | Piece::WK | Piece::BK => {
                &[UP, DOWN, LEFT, RIGHT, UP + LEFT, UP + RIGHT, DOWN + LEFT, DOWN + RIGHT]
            }
        }
    }

    pub fn is_crawling(&self) -> bool {
        match self {
            Piece::WP | Piece::BP | Piece::WK | Piece::BK | Piece::WN | Piece::BN => false,
            _ => true,
        }
    }
}
