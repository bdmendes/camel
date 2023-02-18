pub const ROW_SIZE: usize = 8;
pub const BOARD_SIZE: usize = ROW_SIZE * ROW_SIZE;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Square(pub u8);

impl From<usize> for Square {
    fn from(index: usize) -> Self {
        Square(index as u8)
    }
}

impl From<Square> for usize {
    fn from(square: Square) -> Self {
        square.0 as usize
    }
}

impl Square {
    pub fn to_algebraic(&self) -> String {
        let mut algebraic = String::new();
        algebraic.push(('a' as u8 + self.0 % ROW_SIZE as u8) as char);
        algebraic.push(('1' as u8 + self.0 / ROW_SIZE as u8) as char);
        algebraic
    }

    pub fn from_algebraic(algebraic: &str) -> Square {
        let mut chars = algebraic.chars();
        let col = chars.next().unwrap_or('a') as u8 - ('a' as u8);
        let row = chars.next().unwrap_or('1') as u8 - ('1' as u8);
        Square(row * ROW_SIZE as u8 + col)
    }

    pub fn from_row_col(row: usize, col: usize) -> Square {
        (row * ROW_SIZE + col).into()
    }

    pub fn row(&self) -> usize {
        (self.0 / ROW_SIZE as u8) as usize
    }

    pub fn col(&self) -> usize {
        (self.0 % ROW_SIZE as u8) as usize
    }
}
