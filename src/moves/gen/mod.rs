pub mod pawns;

#[derive(Copy, Clone)]
pub enum MoveStage {
    All,
    CapturesAndPromotions,
    Quiet,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{moves::Move, position::Position};

    use super::MoveStage;

    pub fn assert_eq_vec_move(moves: &[Move], expected: &[&str]) {
        assert_eq!(moves.len(), expected.len());
        moves
            .iter()
            .map(|m| m.to_string())
            .for_each(|m| assert!(expected.contains(&m.as_str())));
    }

    pub fn test_staged(
        position: &str,
        function: fn(&Position, MoveStage, &mut Vec<Move>),
        expected: [Vec<&str>; 3],
    ) {
        let position = Position::from_str(position).unwrap();

        let mut moves1 = Vec::new();
        function(&position, MoveStage::All, &mut moves1);
        moves1.iter().for_each(|m| println!("{}", m));
        assert_eq_vec_move(&moves1, &expected[0]);

        let mut moves2 = Vec::new();
        function(&position, MoveStage::CapturesAndPromotions, &mut moves2);
        assert_eq_vec_move(&moves2, &expected[1]);

        let mut moves3 = Vec::new();
        function(&position, MoveStage::Quiet, &mut moves3);
        assert_eq_vec_move(&moves3, &expected[2]);
    }
}
