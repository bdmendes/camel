use crate::{
    core::{
        moves::{Move, MoveFlag},
        position::{Position, square::Square},
    },
    evaluation::ValueScore,
    search::{Depth, MATE_SCORE, NodeType},
};

pub const DEFAULT_TABLE_SIZE_MB: usize = 256;

const MAX_PLY_DIFF: ValueScore = Depth::MAX as ValueScore;
const NULL_MOVE: Move = Move::new(Square::A1, Square::A1, MoveFlag::Quiet);

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
struct Entry {
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

    pub fn probe(&self, position: &Position, depth: Depth, ply: Depth) -> Option<(ValueScore, NodeType)> {
        // SAFETY: index is always in bounds
        unsafe {
            self.entries
                .get_unchecked(self.index(position))
                .filter(|e| {
                    e.depth >= depth
                        && e.hash_ms16 == position.hash().ms16()
                        && (e.mov == NULL_MOVE || e.mov.pseudo_legal(position))
                })
                .map(|e| {
                    if e.score <= MATE_SCORE + MAX_PLY_DIFF {
                        (e.score + ply as ValueScore, e.node_type)
                    } else if e.score >= -MATE_SCORE - MAX_PLY_DIFF {
                        (e.score - ply as ValueScore, e.node_type)
                    } else {
                        (e.score, e.node_type)
                    }
                })
        }
    }

    pub fn put(
        &mut self,
        position: &Position,
        depth: Depth,
        ply: Depth,
        node_type: NodeType,
        score: ValueScore,
        mov: Option<Move>,
    ) {
        let index = self.index(position);
        // SAFETY: index is always in bounds
        unsafe {
            match self.entries.get_unchecked_mut(index) {
                Some(existing) if existing.depth > depth && node_type != NodeType::PVNode => {}
                slot => {
                    *slot = Some(Entry {
                        score: if score <= MATE_SCORE + MAX_PLY_DIFF {
                            score - ply as ValueScore
                        } else if score >= -MATE_SCORE - MAX_PLY_DIFF {
                            score + ply as ValueScore
                        } else {
                            score
                        },
                        depth,
                        node_type,
                        hash_ms16: position.hash().ms16(),
                        mov: mov.unwrap_or(NULL_MOVE),
                    });
                }
            }
        }
    }

    pub fn hash_move(&self, position: &Position) -> Option<Move> {
        // SAFETY: index is always in bounds
        unsafe {
            self.entries
                .get_unchecked(self.index(position))
                .filter(|e| e.hash_ms16 == position.hash().ms16() && e.mov != NULL_MOVE && e.mov.pseudo_legal(position))
                .map(|m| m.mov)
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
        let position2 = Position::from_str(START_POSITION).unwrap().make_move_str("e2e4").unwrap();
        let mov2 = Move::new(Square::D7, Square::D5, MoveFlag::DoublePawnPush);

        assert_ne!(table.index(&position1), table.index(&position2));

        table.put(&position1, 3, 3, NodeType::PVNode, 100, Some(mov1));
        table.put(&position2, 3, 3, NodeType::PVNode, 200, Some(mov2));

        assert_eq!(table.probe(&position1, 3, 3), Some((100, NodeType::PVNode)));
        assert_eq!(table.probe(&position2, 3, 3), Some((200, NodeType::PVNode)));

        table.clear();
        assert_eq!(table.probe(&position1, 3, 3), None);
        assert_eq!(table.probe(&position2, 3, 3), None);
    }

    #[test]
    fn upsert() {
        let mut table = ScoreTable::new_no_elems(1);
        let position = Position::from_str(START_POSITION).unwrap();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        assert_eq!(table.probe(&position, 4, 4), None);

        // First insertion.
        table.put(&position, 4, 4, NodeType::PVNode, 0, Some(mov));
        assert_eq!(table.probe(&position, 4, 4), Some((0, NodeType::PVNode)));
        assert_eq!(table.probe(&position, 3, 3), Some((0, NodeType::PVNode)));
        assert_eq!(table.probe(&position, 5, 5), None);

        // PV-nodes are always inserted.
        table.put(&position, 3, 3, NodeType::PVNode, 30, Some(mov));
        assert_eq!(table.probe(&position, 4, 4), None);
        assert_eq!(table.probe(&position, 3, 3), Some((30, NodeType::PVNode)));

        // Other nodes are only inserted if the depth is higher or equal.
        table.put(&position, 2, 2, NodeType::AllNode, 0, Some(mov));
        assert_eq!(table.probe(&position, 2, 2), Some((30, NodeType::PVNode)));
        table.put(&position, 3, 3, NodeType::AllNode, 0, Some(mov));
        assert_eq!(table.probe(&position, 3, 3), Some((0, NodeType::AllNode)));
    }

    #[test]
    fn collision_index() {
        let mut table = ScoreTable::new_no_elems(1);
        let position1 = Position::from_str(START_POSITION).unwrap();
        let mov1 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let position2 = Position::from_str(START_POSITION).unwrap().make_move_str("e2e4").unwrap();
        let mov2 = Move::new(Square::D7, Square::D5, MoveFlag::DoublePawnPush);

        table.put(&position1, 3, 3, NodeType::PVNode, 0, Some(mov1));
        assert!(table.probe(&position2, 3, 3).is_none());

        table.put(&position2, 3, 3, NodeType::PVNode, 0, Some(mov2));
        assert!(table.probe(&position1, 3, 3).is_none());
    }

    #[test]
    fn collision_hash() {
        // Let's simulate a 16-bit MSB + modulo index hash collision by inserting a position
        // with the same hash but a move that is trivially invalid.
        let mut table = ScoreTable::new_no_elems(1);
        let position = Position::from_str(START_POSITION).unwrap();
        let valid_mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);
        let invalid_mov = Move::new(Square::E3, Square::E4, MoveFlag::DoublePawnPush);

        table.put(&position, 3, 3, NodeType::PVNode, 0, Some(invalid_mov));
        assert!(table.probe(&position, 3, 3).is_none());

        table.put(&position, 3, 3, NodeType::PVNode, 0, Some(valid_mov));
        assert!(table.probe(&position, 3, 3).is_some());
    }

    #[test]
    fn mate_scoring() {
        let position1 = Position::from_str("3k4/8/7R/6R1/8/8/8/4K3 w - - 0 1").unwrap();
        let mov1 = Move::new(Square::G5, Square::G7, MoveFlag::Quiet);

        let position2 = position1.make_move(mov1);
        let mov2 = Move::new(Square::D8, Square::E8, MoveFlag::Quiet);

        let position3 = position2.make_move(mov2);
        let mov3 = Move::new(Square::H6, Square::H8, MoveFlag::Quiet);

        let mut table = ScoreTable::new_no_elems(100);

        // Position 4 (after mov3) is mated at ply 3. It will yield MATE_SCORE + 3.
        // Position 3 sees mate in 1, and simply inserts the negated child value.
        // Our table adjusts the value so that it is ply-independent.
        table.put(&position3, 1, 2, NodeType::PVNode, -(MATE_SCORE + 3), Some(mov3));

        assert_eq!(table.probe(&position3, 1, 2).unwrap().0, -(MATE_SCORE + 3));
        assert_eq!(table.probe(&position3, 1, 3).unwrap().0, -(MATE_SCORE + 4));
        assert_eq!(table.probe(&position3, 1, 1).unwrap().0, -(MATE_SCORE + 2));

        // At position 2, we are at the mated side.
        table.put(&position2, 2, 1, NodeType::PVNode, MATE_SCORE + 3, Some(mov2));
        assert_eq!(table.probe(&position2, 2, 1).unwrap().0, MATE_SCORE + 3);
        assert_eq!(table.probe(&position2, 2, 2).unwrap().0, MATE_SCORE + 4);
        assert_eq!(table.probe(&position2, 2, 0).unwrap().0, MATE_SCORE + 2);

        // At position 1, we find a mate in 3.
        table.put(&position1, 3, 0, NodeType::PVNode, -(MATE_SCORE + 3), Some(mov1));
        assert_eq!(table.probe(&position1, 3, 0).unwrap().0, -MATE_SCORE - 3);

        // Depth is irrelevant.
        assert_eq!(table.probe(&position1, 2, 0).unwrap().0, -MATE_SCORE - 3);
        assert_eq!(table.probe(&position1, 1, 0).unwrap().0, -MATE_SCORE - 3);
    }

    #[test]
    fn hash_move() {
        let mut table = ScoreTable::new_no_elems(1);
        let position = Position::from_str(START_POSITION).unwrap();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::DoublePawnPush);

        table.put(&position, 3, 3, NodeType::PVNode, 0, Some(mov));
        assert_eq!(table.hash_move(&position), Some(mov));

        let position2 = position.make_move_str("e2e4").unwrap();
        let mov2 = Move::new(Square::D7, Square::D5, MoveFlag::Quiet);

        table.put(&position2, 3, 3, NodeType::PVNode, 0, Some(mov2));
        assert_eq!(table.hash_move(&position), None);
    }

    #[test]
    fn null_move() {
        let mut table = ScoreTable::new_no_elems(1);
        let position = Position::from_str(START_POSITION).unwrap();

        table.put(&position, 3, 3, NodeType::PVNode, 0, None);
        assert_eq!(table.hash_move(&position), None);
        assert!(table.probe(&position, 3, 3).is_some());
    }
}
