use super::{table::SearchTable, Depth};
use crate::{
    evaluation::{moves::evaluate_move, Evaluable, ValueScore},
    moves::Move,
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

#[derive(Debug, PartialEq, Copy, Clone)]
enum MoveStage {
    HashMove,
    All,
}

fn find_next_max_and_swap(moves: &mut ScoredVec<Move>, index: &mut usize) -> Option<PickResult> {
    if index >= &mut moves.len() {
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
    position: Position,
    stage: MoveStage,
    table: Option<Arc<Mutex<SearchTable>>>,
    depth: Option<Depth>,
}

impl MovePicker<true> {
    pub fn new(position: &Position, is_check: bool) -> Self {
        let moves = if is_check { position.moves::<false>() } else { position.moves::<true>() };
        Self {
            index: 0,
            moves: decorate_moves_with_score(&moves, |mov| evaluate_move(position, mov)),
            position: *position,
            table: None,
            depth: None,
            stage: MoveStage::All,
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
        let mut moves = ScoredVec::new();
        if let Some(hash_move) = table.lock().unwrap().get_hash_move(position) {
            moves.push((hash_move, ValueScore::MAX));
        }

        Self {
            index: 0,
            moves,
            position: *position,
            table: Some(table),
            depth: Some(depth),
            stage: MoveStage::HashMove,
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
                self.stage = MoveStage::All;
                let killers =
                    self.table.as_ref().unwrap().lock().unwrap().get_killers(self.depth.unwrap());
                self.moves = decorate_moves_with_score(&self.position.moves::<false>(), |mov| {
                    if killers[0] == Some(mov) || killers[1] == Some(mov) {
                        return Piece::Pawn.value();
                    }
                    evaluate_move(&self.position, mov)
                });

                self.index = 0;
                self.next()
            }
            _ => None,
        }
    }
}
