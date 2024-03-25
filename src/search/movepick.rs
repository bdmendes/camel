use super::{table::SearchTable, Depth};
use crate::{
    evaluation::{moves::evaluate_move, Evaluable, ValueScore},
    moves::{gen::MoveStage, Move},
    position::{board::Piece, Position},
};
use std::sync::{Arc, Mutex};

type ScoredVec<Move> = Vec<(Move, ValueScore)>;
type PickResult = (Move, ValueScore);

fn decorate_moves_with_score<F>(moves: &[Move], f: F) -> ScoredVec<Move>
where
    F: Fn(Move) -> ValueScore,
{
    moves.iter().map(|mov| (*mov, f(*mov))).collect()
}

fn find_next_max_and_swap(moves: &mut ScoredVec<Move>, index: &mut usize) -> Option<PickResult> {
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
    Some((moves[*index - 1].0, moves[*index - 1].1))
}

pub struct MovePicker<const QUIESCE: bool> {
    index: usize,
    moves: ScoredVec<Move>,
    stage: MoveStage,
    position: Position,
    table: Option<Arc<Mutex<SearchTable>>>,
    depth: Option<Depth>,
}

impl MovePicker<true> {
    pub fn new(position: &Position, is_check: bool) -> Self {
        let moves = position.moves(if is_check {
            MoveStage::All
        } else {
            MoveStage::CapturesAndPromotions
        });
        Self {
            index: 0,
            moves: decorate_moves_with_score(&moves, |mov| evaluate_move(position, mov)),
            stage: MoveStage::CapturesAndPromotions,
            position: *position,
            table: None,
            depth: None,
        }
    }
}

impl std::iter::Iterator for MovePicker<true> {
    type Item = PickResult;

    fn next(&mut self) -> Option<Self::Item> {
        find_next_max_and_swap(&mut self.moves, &mut self.index)
    }
}

impl MovePicker<false> {
    pub fn new(position: &Position, table: Arc<Mutex<SearchTable>>, depth: Depth) -> Self {
        let mut moves = ScoredVec::with_capacity(1);
        if let Some(hash_move) = table.lock().unwrap().get_hash_move(position) {
            moves.push((hash_move, ValueScore::MAX));
        }

        Self {
            index: 0,
            moves,
            stage: MoveStage::HashMove,
            position: *position,
            table: Some(table),
            depth: Some(depth),
        }
    }
}

impl std::iter::Iterator for MovePicker<false> {
    type Item = PickResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((mov, score)) = find_next_max_and_swap(&mut self.moves, &mut self.index) {
            return Some((mov, score));
        }

        match self.stage {
            MoveStage::HashMove => {
                self.stage = MoveStage::CapturesAndPromotions;
                self.moves = decorate_moves_with_score(
                    &self.position.moves(MoveStage::CapturesAndPromotions),
                    |mov| evaluate_move(&self.position, mov),
                );

                self.index = 0;
                self.next()
            }
            MoveStage::CapturesAndPromotions => {
                self.stage = MoveStage::NonCaptures;
                let all_non_capture_moves = self.position.moves(MoveStage::NonCaptures);

                let killers =
                    self.table.as_ref().unwrap().lock().unwrap().get_killers(self.depth.unwrap());
                self.moves = decorate_moves_with_score(&all_non_capture_moves, |mov| {
                    if killers[1] == Some(mov) || killers[0] == Some(mov) {
                        Piece::Queen.value()
                    } else {
                        evaluate_move(&self.position, mov)
                    }
                });

                self.index = 0;
                self.next()
            }
            _ => None,
        }
    }
}
