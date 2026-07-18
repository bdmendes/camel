use crate::moves::Move;

const MAX_SIZE: usize = 128;

pub struct MoveVec {
    data: [(Option<Move>, i8); MAX_SIZE],
    raw_size: usize,
    length: usize,
}

impl Default for MoveVec {
    fn default() -> Self {
        Self {
            data: [(None, i8::MIN); MAX_SIZE],
            raw_size: 0,
            length: 0,
        }
    }
}

pub struct MoveVecIterator<'a> {
    move_list: &'a mut MoveVec,
    index: usize,
}

impl<'a> Iterator for MoveVecIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut best_score = self.move_list.data[self.index].1;
        let mut currently_empty = self.move_list.data[self.index].0.is_none();
        let mut curr_idx = self.index + 1;

        // Put the best move at the front without sorting the entire vector.
        // This laziness fits the minimax model where most moves can be discarded.
        while curr_idx < self.move_list.raw_size {
            let (mov, score) = self.move_list.data[curr_idx];
            if mov.is_some() && (score > best_score || currently_empty) {
                best_score = score;
                self.move_list.data.swap(self.index, curr_idx);
                currently_empty = false;
            }
            curr_idx += 1;
        }

        self.index += 1;
        self.move_list.data[self.index - 1].0
    }
}

impl<'a> IntoIterator for &'a mut MoveVec {
    type Item = Move;
    type IntoIter = MoveVecIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl MoveVec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn clear(&mut self) {
        self.raw_size = 0;
        self.length = 0;
    }

    pub fn push(&mut self, mov: Move) {
        self.push_scored(mov, i8::MIN);
    }

    pub fn push_scored(&mut self, mov: Move, score: i8) {
        if self.raw_size < MAX_SIZE {
            self.data[self.raw_size] = (Some(mov), score);
            self.raw_size += 1;
            self.length += 1;
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> MoveVecIterator<'a> {
        MoveVecIterator {
            move_list: self,
            index: 0,
        }
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(Move) -> bool,
    {
        for i in 0..self.raw_size {
            if let Some(mov) = self.data[i].0
                && !f(mov)
            {
                self.data[i] = (None, i8::MIN);
                self.length -= 1;
            }
        }
    }

    pub fn evaluate<F>(&mut self, f: F)
    where
        F: Fn(Move) -> i8,
    {
        for i in 0..self.raw_size {
            if let Some(mov) = self.data[i].0 {
                self.data[i].1 = f(mov);
            }
        }
    }

    pub fn contains(&self, mov: Move) -> bool {
        for i in 0..self.raw_size {
            if let Some(m) = self.data[i].0
                && m == mov
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{Move, MoveFlag, vec::MoveVec},
        position::square::Square,
    };

    #[test]
    fn push_len_clear() {
        let mut list = MoveVec::default();
        assert_eq!(list.len(), 0);
        list.push(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush));
        list.push(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush));
        assert_eq!(list.len(), 2);
        list.clear();
        assert_eq!(list.len(), 0);
        list.push(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn iter() {
        let mut list = MoveVec::default();
        list.push_scored(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush), 0);
        list.push_scored(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush), 2);
        assert_eq!(list.data[0].0, Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush)));

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush)));
        assert_eq!(iter.next(), Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush)));
        assert_eq!(iter.next(), None);
        assert_eq!(list.data[0].0, Some(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush)));

        list.data[0] = (None, i8::MIN);
        let mut iter2 = list.iter_mut();
        assert_eq!(iter2.next(), Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush)));
        assert_eq!(iter2.next(), None);
    }

    #[test]
    pub fn retain_iter() {
        let mut list = MoveVec::default();
        list.push_scored(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush), 0);
        list.push_scored(Move::new(Square::G1, Square::F3, MoveFlag::Quiet), 2);
        list.push_scored(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush), 4);
        list.push_scored(Move::new(Square::B8, Square::C6, MoveFlag::Quiet), 6);
        assert_eq!(list.len(), 4);
        assert_eq!(list.raw_size, 4);

        list.retain(|m| m.flag() == MoveFlag::Quiet);
        assert_eq!(list.len(), 2);
        assert_eq!(list.raw_size, 4);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(Move::new(Square::B8, Square::C6, MoveFlag::Quiet)));
        assert_eq!(iter.next(), Some(Move::new(Square::G1, Square::F3, MoveFlag::Quiet)));
        assert_eq!(iter.next(), None);

        assert_eq!(list.len(), 2);
        assert_eq!(list.raw_size, 4);

        assert_eq!(list.data[0].0, Some(Move::new(Square::B8, Square::C6, MoveFlag::Quiet)));
        assert_eq!(list.data[1].0, Some(Move::new(Square::G1, Square::F3, MoveFlag::Quiet)));
        assert_eq!(list.data[2].0, None);
        assert_eq!(list.data[3].0, None);
    }

    #[test]
    fn evaluate_iter() {
        let mut list = MoveVec::default();
        list.push(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush));
        list.push(Move::new(Square::G1, Square::F3, MoveFlag::Quiet));

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush)));
        assert_eq!(iter.next(), Some(Move::new(Square::G1, Square::F3, MoveFlag::Quiet)));
        assert_eq!(iter.next(), None);

        list.evaluate(|m| match m.flag() {
            MoveFlag::DoublePawnPush => 1,
            MoveFlag::Quiet => 2,
            _ => 0,
        });

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(Move::new(Square::G1, Square::F3, MoveFlag::Quiet)));
        assert_eq!(iter.next(), Some(Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn contains() {
        let mut list = MoveVec::default();
        let mov1 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let mov2 = Move::new(Square::G1, Square::F3, MoveFlag::Quiet);
        list.push(mov1);
        list.push(mov2);

        assert!(list.contains(mov1));
        assert!(list.contains(mov2));
        assert!(!list.contains(Move::new(Square::E7, Square::E5, MoveFlag::DoublePawnPush)));
    }
}
