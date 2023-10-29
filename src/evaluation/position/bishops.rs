use crate::{
    evaluation::ValueScore,
    position::{bitboard::Bitboard, board::Piece, square::Square, Color, Position},
};

const BISHOP_PAIR_BONUS: ValueScore = 30;
const SAME_COLOR_CENTER_PAWN_PENALTY: ValueScore = -10;
const CENTER_PAWNS: Bitboard = Bitboard::new(
    1 << Square::E4 as u64
        | 1 << Square::D4 as u64
        | 1 << Square::E5 as u64
        | 1 << Square::D5 as u64,
);

pub fn evaluate_bishops(position: &Position) -> ValueScore {
    let mut score = 0;

    for color in Color::list() {
        let our_bishops =
            position.board.pieces_bb(Piece::Bishop) & position.board.occupancy_bb(*color);

        // Bishop pair
        if our_bishops.count_ones() > 1 {
            score += BISHOP_PAIR_BONUS * color.sign();
        }

        // Bad bishop penalty
        for bishop in our_bishops {
            let bishop_color = bishop.color();
            let our_central_pawns = CENTER_PAWNS
                & position.board.pieces_bb(Piece::Pawn)
                & position.board.occupancy_bb(*color);
            score += SAME_COLOR_CENTER_PAWN_PENALTY
                * our_central_pawns.color_squares(bishop_color).count_ones() as ValueScore
                * color.sign();
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::position::bishops::{BISHOP_PAIR_BONUS, SAME_COLOR_CENTER_PAWN_PENALTY},
        position::Position,
    };

    #[test]
    fn bishop_pair() {
        let position = Position::from_fen(
            "1qr1kr1b/p1p1ppp1/4nn2/1p6/2p5/1P2NQ2/P2PPPPP/B1R1KR1B b KQkq - 0 7",
        )
        .unwrap();
        let bishops_score = super::evaluate_bishops(&position);
        assert_eq!(bishops_score, BISHOP_PAIR_BONUS);
    }

    #[test]
    fn bad_bishop() {
        let position =
            Position::from_fen("r2qkb1r/ppp2ppp/2n1p3/3n4/3P4/2PQ1N2/PP3PPP/RNB1K2R w KQkq - 0 8")
                .unwrap();
        let bishops_score = super::evaluate_bishops(&position);
        assert_eq!(bishops_score, SAME_COLOR_CENTER_PAWN_PENALTY);
    }
}
