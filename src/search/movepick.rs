use super::{table::SearchTable, Depth};
use crate::{
    evaluation::{moves::evaluate_move, Evaluable, ValueScore},
    moves::{gen::MoveStage, Move},
    position::{board::Piece, Position},
};
use rand::{thread_rng, Rng};
use std::sync::Arc;

type ScoredVec<Move> = Vec<(Move, ValueScore)>;

const RANDOM_FACTOR: ValueScore = 1000;

pub struct MovePicker<const QUIESCE: bool> {
    index: usize,
    moves: ScoredVec<Move>,
    stage: MoveStage,
    position: Position,
    table: Option<Arc<SearchTable>>,
    depth: Option<Depth>,
}

impl MovePicker<true> {
    pub fn new(position: &Position, is_check: bool) -> Self {
        let stage = if is_check { MoveStage::All } else { MoveStage::CapturesAndPromotions };
        let moves = position.moves(stage);
        Self {
            index: 0,
            moves: decorate_moves_with_score(&moves, |mov| evaluate_move(position, mov)),
            stage,
            position: *position,
            table: None,
            depth: None,
        }
    }

    pub fn empty(&self) -> bool {
        self.moves.is_empty()
    }
}

impl std::iter::Iterator for MovePicker<true> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        find_next_max_and_swap(&mut self.moves, &mut self.index)
    }
}

impl MovePicker<false> {
    pub fn new(position: &Position, table: Arc<SearchTable>, depth: Depth, shuffle: bool) -> Self {
        let moves = if !shuffle {
            if let Some(hash_move) = table.get_hash_move(position) {
                vec![(hash_move, ValueScore::MAX)]
            } else {
                vec![]
            }
        } else {
            position
                .moves(MoveStage::All)
                .into_iter()
                .map(|m| (m, thread_rng().gen_range(0..RANDOM_FACTOR)))
                .collect::<Vec<_>>()
        };

        Self {
            index: 0,
            moves,
            stage: if !shuffle { MoveStage::HashMove } else { MoveStage::All },
            position: *position,
            table: Some(table),
            depth: Some(depth),
        }
    }

    pub fn empty(&mut self) -> bool {
        loop {
            if !self.moves.is_empty() {
                return false;
            }
            self.advance_stage();
            if self.stage == MoveStage::All {
                return true;
            }
        }
    }

    fn advance_stage(&mut self) {
        match self.stage {
            MoveStage::HashMove => {
                self.stage = MoveStage::CapturesAndPromotions;
                self.moves = decorate_moves_with_score(
                    &self.position.moves(MoveStage::CapturesAndPromotions),
                    |mov| evaluate_move(&self.position, mov),
                );
            }
            MoveStage::CapturesAndPromotions => {
                self.stage = MoveStage::NonCaptures;
                let all_non_capture_moves = self.position.moves(MoveStage::NonCaptures);

                let killers = self.table.as_ref().unwrap().get_killers(self.depth.unwrap());
                self.moves = decorate_moves_with_score(&all_non_capture_moves, |mov| {
                    if killers[1] == Some(mov) || killers[0] == Some(mov) {
                        Piece::Queen.value()
                    } else {
                        evaluate_move(&self.position, mov)
                    }
                });
            }
            _ => {
                self.stage = MoveStage::All;
            }
        }

        self.index = 0;
    }
}

impl std::iter::Iterator for MovePicker<false> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mov) = find_next_max_and_swap(&mut self.moves, &mut self.index) {
            return Some(mov);
        }

        if self.stage == MoveStage::All {
            return None;
        }

        self.advance_stage();
        self.next()
    }
}

fn decorate_moves_with_score<F>(moves: &[Move], f: F) -> ScoredVec<Move>
where
    F: Fn(Move) -> ValueScore,
{
    moves.iter().map(|mov| (*mov, f(*mov))).collect()
}

fn find_next_max_and_swap(moves: &mut ScoredVec<Move>, index: &mut usize) -> Option<Move> {
    if *index >= moves.len() {
        return None;
    }

    let mut best_score = moves[*index].1;

    for i in (*index + 1)..moves.len() {
        if moves[i].1 > best_score {
            best_score = moves[i].1;
            moves.swap(i, *index);
        }
    }

    *index += 1;
    Some(moves[*index - 1].0)
}
