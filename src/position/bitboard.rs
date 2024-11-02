use super::Square;

#[derive(Default)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn is_set(&self, square: Square) -> bool {
        (self.0 & (1 << square as u64)) != 0
    }

    pub fn set(&mut self, square: Square) {
        self.0 |= square as u64;
    }

    pub fn clear(&mut self, square: Square) {
        self.0 &= !(square as u64);
    }
}
