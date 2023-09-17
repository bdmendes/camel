use super::{table::SearchTable, Depth};
use crate::{
    evaluation::{moves::evaluate_move, Evaluable, ValueScore},
    moves::Move,
    position::{board::Piece, Position},
};
use std::sync::{Arc, Mutex};

type ScoredVec<Move> = Vec<(Move, ValueScore)>;
type PickResult = (Move, ValueScore, usize);

fn decorate_moves_with_score<F>(moves: &[Move], f: F) -> ScoredVec<Move>
where
    F: Fn(Move) -> ValueScore,
{
    moves.iter().map(|mov| (*mov, f(*mov))).collect()
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
    Some((moves[*index - 1].0, moves[*index - 1].1, *index - 1))
}

enum MoveStage {
    HashMove,
    Captures,
    Others,
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
        let moves = position.moves(!is_check);
        Self {
            index: 0,
            moves: decorate_moves_with_score(&moves, |mov| evaluate_move(position, mov)),
            stage: MoveStage::Captures,
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
        let moves = if let Some(hash_move) = table.lock().unwrap().get_hash_move(position) {
            vec![(hash_move, ValueScore::MAX)]
        } else {
            vec![]
        };

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
        if let Some((mov, score, index)) = find_next_max_and_swap(&mut self.moves, &mut self.index)
        {
            return Some((mov, score, index));
        }

        match self.stage {
            MoveStage::HashMove => {
                self.stage = MoveStage::Captures;
                self.moves = decorate_moves_with_score(&self.position.moves(true), |mov| {
                    evaluate_move(&self.position, mov)
                });

                self.index = 0;
                self.next()
            }
            MoveStage::Captures => {
                self.stage = MoveStage::Others;
                let all_non_capture_moves = self
                    .position
                    .moves(false)
                    .into_iter()
                    .filter(|mov| !mov.flag().is_capture())
                    .collect::<Vec<_>>();

                let killers =
                    self.table.as_ref().unwrap().lock().unwrap().get_killers(self.depth.unwrap());
                self.moves = decorate_moves_with_score(&all_non_capture_moves, |mov| {
                    if killers[0] == Some(mov) || killers[1] == Some(mov) {
                        Piece::Queen.value()
                    } else {
                        evaluate_move(&self.position, mov)
                    }
                });

                self.index = 0;
                self.next()
            }
            MoveStage::Others => None,
        }
    }
}
