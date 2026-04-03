use crate::{moves::Move, search::MAX_DEPTH};

pub struct KillerTable {
    killers: [Option<Move>; 2 * (MAX_DEPTH + 1) as usize],
}

impl Default for KillerTable {
    fn default() -> Self {
        Self {
            killers: [None; 2 * (MAX_DEPTH + 1) as usize],
        }
    }
}

impl KillerTable {
    pub fn get(&self, ply: u8) -> [Option<Move>; 2] {
        let index = ply.min(MAX_DEPTH) as usize * 2;
        [self.killers[index], self.killers[index + 1]]
    }

    pub fn put(&mut self, ply: u8, mov: Move) {
        let index = ply.min(MAX_DEPTH) as usize * 2;
        if self.killers[index] != Some(mov) {
            self.killers[index + 1] = self.killers[index];
            self.killers[index] = Some(mov);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{moves::MoveFlag, position::square::Square};

    #[test]
    fn put_get() {
        let mut table = KillerTable::default();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mov2 = Move::new(Square::D2, Square::D4, MoveFlag::DoublePawnPush);

        assert_eq!(table.get(0), [None, None]);

        table.put(0, mov);
        assert_eq!(table.get(0), [Some(mov), None]);

        table.put(0, mov2);
        assert_eq!(table.get(0), [Some(mov2), Some(mov)]);
    }

    #[test]
    fn repeated_put() {
        let mut table = KillerTable::default();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        table.put(0, mov);
        table.put(0, mov);
        assert_eq!(table.get(0), [Some(mov), None]);
    }

    #[test]
    fn override_put() {
        let mut table = KillerTable::default();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mov2 = Move::new(Square::D2, Square::D4, MoveFlag::DoublePawnPush);
        let mov3 = Move::new(Square::C2, Square::C4, MoveFlag::DoublePawnPush);

        table.put(0, mov);
        table.put(0, mov2);
        table.put(0, mov3);
        assert_eq!(table.get(0), [Some(mov3), Some(mov2)]);
    }

    #[test]
    fn multiple_plies() {
        let mut table = KillerTable::default();
        let mov1 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mov2 = Move::new(Square::D2, Square::D4, MoveFlag::DoublePawnPush);
        let mov3 = Move::new(Square::C2, Square::C4, MoveFlag::DoublePawnPush);
        let mov4 = Move::new(Square::B2, Square::B4, MoveFlag::DoublePawnPush);

        table.put(0, mov1);
        table.put(1, mov2);
        table.put(0, mov3);
        table.put(1, mov4);

        assert_eq!(table.get(0), [Some(mov3), Some(mov1)]);
        assert_eq!(table.get(1), [Some(mov4), Some(mov2)]);
    }

    #[test]
    fn max_ply_boundary() {
        let mut table = KillerTable::default();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        table.put(MAX_DEPTH, mov);
        assert_eq!(table.get(MAX_DEPTH), [Some(mov), None]);

        let mov2 = Move::new(Square::D2, Square::D4, MoveFlag::DoublePawnPush);
        table.put(MAX_DEPTH + 1, mov2);
        assert_eq!(table.get(MAX_DEPTH + 1), [Some(mov2), Some(mov)]);
    }
}
