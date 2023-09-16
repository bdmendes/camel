use crate::{
    evaluation::ValueScore,
    position::{board::Piece, square::Square, Color, Position},
};

fn king_tropism(position: &Position, king_color: Color, king_square: Square) -> ValueScore {
    let them_occupancy = position.board.occupancy_bb(king_color.opposite())
        & !position.board.pieces_bb(Piece::Pawn)
        & !position.board.pieces_bb(Piece::King);

    let tropism = them_occupancy.fold(0, |acc, sq| {
        let distance = sq.distance(king_square);
        let piece_cof = match position.board.piece_at(sq) {
            Some(Piece::Queen) => 3,
            Some(Piece::Rook) => 2,
            Some(Piece::Bishop) | Some(Piece::Knight) => 1,
            _ => unreachable!(),
        };
        acc + ((14 - distance) * piece_cof) as ValueScore
    });

    -tropism
}

pub fn evaluate_king_safety(position: &Position, midgame_ratio: u8) -> ValueScore {
    let white_king_square = (position.board.occupancy_bb(Color::White)
        & position.board.pieces_bb(Piece::King))
    .into_iter()
    .next();
    let black_king_square = (position.board.occupancy_bb(Color::Black)
        & position.board.pieces_bb(Piece::King))
    .into_iter()
    .next();

    if white_king_square.is_none() || black_king_square.is_none() {
        return 0;
    }

    let mut score = 0;

    score += king_tropism(position, Color::White, white_king_square.unwrap())
        * midgame_ratio as ValueScore
        / 255;
    score -= king_tropism(position, Color::Black, black_king_square.unwrap())
        * midgame_ratio as ValueScore
        / 255;

    score
}

#[cfg(test)]
mod tests {
    use super::king_tropism;
    use crate::{
        evaluation::ValueScore,
        position::{board::Piece, fen::START_FEN, Color, Position},
    };

    fn position_tropism(position: &Position) -> ValueScore {
        let white_king_square = (position.board.occupancy_bb(Color::White)
            & position.board.pieces_bb(Piece::King))
        .into_iter()
        .next()
        .unwrap();
        let black_king_square = (position.board.occupancy_bb(Color::Black)
            & position.board.pieces_bb(Piece::King))
        .into_iter()
        .next()
        .unwrap();

        king_tropism(position, Color::White, white_king_square)
            - king_tropism(position, Color::Black, black_king_square)
    }

    #[test]
    fn tropism_smoke() {
        let start_position = Position::from_fen(START_FEN).unwrap();
        assert_eq!(position_tropism(&start_position), 0);
    }

    #[test]
    fn tropism_1() {
        let position =
            Position::from_fen("r5k1/2qb1p1p/5QpB/ppbpr3/2pN4/2P3P1/PP3P1P/3RR1K1 b - - 1 21")
                .unwrap();
        assert!(position_tropism(&position) > 20);
    }
}
