use crate::{
    evaluation::ValueScore,
    position::{board::Piece, Color, Position},
};

const BISHOP_PAIR_BONUS: ValueScore = 30;

pub fn evaluate_bishops(position: &Position) -> ValueScore {
    let mut score = 0;

    for color in Color::list() {
        let our_bishops =
            position.board.pieces_bb(Piece::Bishop) & position.board.occupancy_bb(*color);

        // Bishop pair
        if our_bishops.count_ones() > 1 {
            score += BISHOP_PAIR_BONUS * color.sign();
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{evaluation::position::bishops::BISHOP_PAIR_BONUS, position::Position};

    #[test]
    fn bishop_pair() {
        let position = Position::from_fen(
            "1qr1kr1b/p1p1ppp1/4nn2/1p6/2p5/1P2NQ2/P2PPPPP/B1R1KR1B b KQkq - 0 7",
        )
        .unwrap();
        let bishops_score = super::evaluate_bishops(&position);
        assert_eq!(bishops_score, BISHOP_PAIR_BONUS);
    }
}