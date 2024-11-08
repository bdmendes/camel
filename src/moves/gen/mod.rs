pub mod pawns;

pub enum MoveStage {
    All,
    CapturesAndPromotions,
    Quiet,
}

#[cfg(test)]
mod tests {
    use crate::moves::Move;

    pub fn assert_eq_vec_move(moves: &[Move], expected: &[&str]) {
        assert_eq!(moves.len(), expected.len());
        moves
            .iter()
            .map(|m| m.to_string())
            .for_each(|m| assert!(expected.contains(&m.as_str())));
    }
}
