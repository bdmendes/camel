use crate::{
    core::{color::Color, piece::Piece, Position},
    evaluation::ValueScore,
};

pub static mut BISHOP_PAIR_BONUS: ValueScore = 49;

pub fn evaluate_bishops(position: &Position) -> ValueScore {
    let mut score = 0;

    for color in Color::list() {
        let our_bishops = position.pieces_color_bb(Piece::Bishop, *color);
        if our_bishops.count_ones() > 1 {
            score += unsafe { BISHOP_PAIR_BONUS * color.sign() };
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{core::Position, evaluation::position::bishops::BISHOP_PAIR_BONUS};

    #[test]
    fn bishop_pair() {
        let position = Position::from_str(
            "1qr1kr1b/p1p1ppp1/4nn2/1p6/2p5/1P2NQ2/P2PPPPP/B1R1KR1B b KQkq - 0 7",
        )
        .unwrap();
        let bishops_score = super::evaluate_bishops(&position);
        assert_eq!(bishops_score, unsafe { BISHOP_PAIR_BONUS });
    }
}
