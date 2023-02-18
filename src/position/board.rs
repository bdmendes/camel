use super::{Piece, Square};

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Board(pub [Option<Piece>; 64]);

impl std::ops::Index<Square> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: Square) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl std::ops::IndexMut<Square> for Board {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl std::ops::Index<usize> for Board {
    type Output = Option<Piece>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
