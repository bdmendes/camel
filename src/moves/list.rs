use crate::moves::Move;

pub struct MoveList {
    data: [(Option<Move>, i8); 256],
    raw_size: usize,
    length: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            data: [(None, i8::MIN); 256],
            raw_size: 0,
            length: 0,
        }
    }
}

pub struct MoveListIterator<'a> {
    move_list: &'a mut MoveList,
    index: usize,
}

impl<'a> Iterator for MoveListIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut best_score = self.move_list.data[self.index].1;
        let mut curr_idx = self.index + 1;

        // Put the best move at the front without sorting the entire list.
        // This laziness fits the minimax model where most moves can be discarded.
        while curr_idx < self.move_list.raw_size {
            let score = self.move_list.data[curr_idx].1;
            if score > best_score {
                best_score = score;
                self.move_list.data.swap(self.index, curr_idx);
            }
            curr_idx += 1;
        }

        self.index += 1;
        self.move_list.data[self.index - 1].0
    }
}

impl<'a> IntoIterator for &'a mut MoveList {
    type Item = Move;
    type IntoIter = MoveListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl MoveList {
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

    fn push_internal(&mut self, mov: Move, score: Option<i8>) {
        self.data[self.raw_size] = (Some(mov), score.unwrap_or(i8::MIN));
        self.raw_size += 1;
        self.length += 1;
    }

    pub fn push(&mut self, mov: Move) {
        self.push_internal(mov, None);
    }

    pub fn push_scored(&mut self, mov: Move, score: i8) {
        self.push_internal(mov, Some(score));
    }

    pub fn iter_mut<'a>(&'a mut self) -> MoveListIterator<'a> {
        MoveListIterator {
            move_list: self,
            index: 0,
        }
    }

    pub fn retain(&mut self, f: fn(Move) -> bool) {
        for i in 0..self.raw_size {
            if let Some(mov) = self.data[i].0
                && !f(mov)
            {
                self.data[i] = (None, i8::MIN);
                self.length -= 1;
            }
        }
    }

    pub fn evaluate(&mut self, f: fn(Move) -> i8) {
        for i in 0..self.raw_size {
            if let Some(mov) = self.data[i].0 {
                self.data[i].1 = f(mov);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{Move, MoveFlag, list::MoveList},
        position::square::Square,
    };

    #[test]
    fn push_len_clear() {
        let mut list = MoveList::default();
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
        let mut list = MoveList::default();
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
        let mut list = MoveList::default();
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
        let mut list = MoveList::default();
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
}
