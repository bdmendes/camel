use crate::{
    core::{moves::Move, position::Position},
    evaluation::ValueScore,
    search::{Depth, pvs::NodeType},
};

pub const DEFAULT_TABLE_SIZE_MB: usize = 256;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Entry {
    score: ValueScore,
    node_type: NodeType,
    depth: Depth,
    hash_ms16: u16,
    mov: Move,
}

pub struct ScoreTable {
    entries: Vec<Option<Entry>>,
}

impl ScoreTable {
    pub fn new(size_mb: usize) -> Self {
        let mut table = Self { entries: Vec::new() };
        table.resize(size_mb);
        table
    }

    pub fn new_no_elems(no_elems: usize) -> Self {
        let mut table = Self { entries: Vec::new() };
        table.entries.resize(no_elems, None);
        table
    }

    pub fn resize(&mut self, size_mb: usize) {
        let no_elements = size_mb * 1024 * 1024 / std::mem::size_of::<Option<Entry>>();
        self.entries.resize(no_elements, None);
    }

    pub fn clear(&mut self) {
        self.entries.fill(None);
    }

    fn index(&self, position: &Position) -> usize {
        (position.hash().value() as usize) % self.entries.len()
    }

    pub fn probe(&self, position: &Position, depth: Depth) -> Option<Entry> {
        // SAFETY: index is always in bounds
        unsafe {
            self.entries
                .get_unchecked(self.index(position))
                .filter(|e| e.depth >= depth && e.hash_ms16 == position.hash().ms16() && e.mov.pseudo_legal(position))
        }
    }

    pub fn put(&mut self, position: &Position, depth: Depth, node_type: NodeType, score: ValueScore, mov: Move) {
        let index = self.index(position);
        // SAFETY: index is always in bounds
        unsafe {
            match self.entries.get_unchecked_mut(index) {
                Some(existing) if existing.depth > depth && node_type != NodeType::PVNode => {}
                slot => {
                    *slot = Some(Entry {
                        score,
                        depth,
                        node_type,
                        hash_ms16: position.hash().ms16(),
                        mov,
                    });
                }
            }
        }
    }

    pub fn hashfull_millis(&self) -> usize {
        // The table is sparse, so sampling a bit from the start suffices.
        self.entries.iter().take(10_000).filter(|e| e.is_some()).count() / 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        moves::MoveFlag,
        position::{fen::START_POSITION, square::Square},
    };
    use std::str::FromStr;

    #[test]
    fn index_clear() {
        let mut table = ScoreTable::new_no_elems(10);
        let position1 = Position::from_str(START_POSITION).unwrap();
        let mov1 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let position2 = Position::from_str(START_POSITION)
            .unwrap()
            .make_move_str("e2e4")
            .unwrap();
        let mov2 = Move::new(Square::D7, Square::D5, MoveFlag::DoublePawnPush);

        assert_ne!(table.index(&position1), table.index(&position2));

        table.put(&position1, 3, NodeType::PVNode, 100, mov1);
        table.put(&position2, 3, NodeType::PVNode, 200, mov2);

        assert_eq!(
            table.probe(&position1, 3),
            Some(Entry {
                depth: 3,
                score: 100,
                hash_ms16: position1.hash().ms16(),
                node_type: NodeType::PVNode,
                mov: mov1
            })
        );
        assert_eq!(
            table.probe(&position2, 3),
            Some(Entry {
                depth: 3,
                score: 200,
                hash_ms16: position2.hash().ms16(),
                node_type: NodeType::PVNode,
                mov: mov2
            })
        );

        table.clear();
        assert_eq!(table.probe(&position1, 3), None);
        assert_eq!(table.probe(&position2, 3), None);
    }

    #[test]
    fn upsert() {
        let mut table = ScoreTable::new_no_elems(1);
        let position = Position::from_str(START_POSITION).unwrap();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        assert_eq!(table.probe(&position, 4), None);

        let expected = Entry {
            depth: 4,
            score: 0,
            hash_ms16: position.hash().ms16(),
            node_type: NodeType::PVNode,
            mov,
        };

        // First insertion.
        table.put(&position, 4, NodeType::PVNode, 0, mov);
        assert_eq!(table.probe(&position, 4), Some(expected));
        assert_eq!(table.probe(&position, 3), Some(expected));
        assert_eq!(table.probe(&position, 5), None);

        // PV-nodes are always inserted.
        table.put(&position, 3, NodeType::PVNode, 30, mov);
        assert_eq!(table.probe(&position, 4), None);
        assert_eq!(
            table.probe(&position, 3),
            Some(Entry {
                depth: 3,
                score: 30,
                ..expected
            })
        );

        // Other nodes are only inserted if the depth is higher or equal.
        table.put(&position, 2, NodeType::AllNode, 0, mov);
        assert_eq!(
            table.probe(&position, 2),
            Some(Entry {
                depth: 3,
                score: 30,
                ..expected
            })
        );
        table.put(&position, 3, NodeType::AllNode, 0, mov);
        assert_eq!(
            table.probe(&position, 3),
            Some(Entry {
                depth: 3,
                node_type: NodeType::AllNode,
                ..expected
            })
        );
    }

    #[test]
    fn collision() {
        let mut table = ScoreTable::new_no_elems(1);
        let position1 = Position::from_str(START_POSITION).unwrap();
        let mov1 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let position2 = Position::from_str(START_POSITION)
            .unwrap()
            .make_move_str("e2e4")
            .unwrap();
        let mov2 = Move::new(Square::D7, Square::D5, MoveFlag::DoublePawnPush);

        table.put(&position1, 3, NodeType::PVNode, 0, mov1);
        assert_eq!(table.probe(&position1, 3).unwrap().hash_ms16, position1.hash().ms16());
        assert!(table.probe(&position2, 3).is_none());

        table.put(&position2, 3, NodeType::PVNode, 0, mov2);
        assert_eq!(table.probe(&position2, 3).unwrap().hash_ms16, position2.hash().ms16());
        assert!(table.probe(&position1, 3).is_none());
    }
}
