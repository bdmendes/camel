use crate::{
    evaluation::ValueScore,
    position::{bitboard::Bitboard, board::Piece, square::Square, Color, Position},
};

pub static mut SHELTER_PENALTY: ValueScore = -20;

fn king_pawn_shelter(position: &Position, king_color: Color, king_square: Square) -> ValueScore {
    let mut shelter = 0;

    let our_pawns = position.board.pieces_bb_color(Piece::Pawn, king_color);

    let file_min = match king_square.file() {
        0 => 0,
        _ => king_square.file() - 1,
    };
    let file_max = match king_square.file() {
        7 => 7,
        _ => king_square.file() + 1,
    };

    for file in file_min..=file_max {
        let our_pawns_on_file = our_pawns & Bitboard::file_mask(file);
        let most_advanced_pawn = match king_color {
            Color::White => our_pawns_on_file.into_iter().next(),
            Color::Black => our_pawns_on_file.into_iter().next_back(),
        };
        unsafe {
            if let Some(pawn_square) = most_advanced_pawn {
                let rank_diff = (pawn_square.rank() as i8 - king_square.rank() as i8).abs();
                let shelter_penalty = match rank_diff {
                    0 => 0,
                    1 => 0,
                    2 => SHELTER_PENALTY / 2,
                    3 => SHELTER_PENALTY,
                    _ => SHELTER_PENALTY * 2,
                };
                shelter += shelter_penalty;
            } else {
                shelter += SHELTER_PENALTY;
            }
        }
    }

    shelter
}

fn king_tropism(position: &Position, king_color: Color, king_square: Square) -> ValueScore {
    let them_occupancy = position.board.occupancy_bb(king_color.opposite())
        & !position.board.pieces_bb(Piece::Pawn)
        & !position.board.pieces_bb(Piece::King);

    let tropism = them_occupancy.fold(0, |acc, sq| {
        let distance = sq.manhattan_distance(king_square);
        let piece_cof = match position.board.piece_at(sq) {
            Some(Piece::Queen) | Some(Piece::Rook) => 2,
            Some(Piece::Bishop) | Some(Piece::Knight) => 1,
            _ => unreachable!(),
        };
        acc + ((14 - distance) * piece_cof) as ValueScore
    });

    -tropism
}

pub fn evaluate_king_safety(position: &Position, midgame_ratio: u8) -> ValueScore {
    let white_king_square =
        position.board.pieces_bb_color(Piece::King, Color::White).into_iter().next();
    let black_king_square =
        position.board.pieces_bb_color(Piece::King, Color::Black).into_iter().next();

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

    score += king_pawn_shelter(position, Color::White, white_king_square.unwrap())
        * midgame_ratio as ValueScore
        / 255;
    score -= king_pawn_shelter(position, Color::Black, black_king_square.unwrap())
        * midgame_ratio as ValueScore
        / 255;

    score
}

#[cfg(test)]
mod tests {
    use super::king_tropism;
    use crate::{
        evaluation::ValueScore,
        position::{
            board::Piece,
            fen::{FromFen, START_FEN},
            Color, Position,
        },
    };

    fn position_tropism(position: &Position) -> ValueScore {
        let white_king_square =
            position.board.pieces_bb_color(Piece::King, Color::White).into_iter().next().unwrap();
        let black_king_square =
            position.board.pieces_bb_color(Piece::King, Color::Black).into_iter().next().unwrap();

        king_tropism(position, Color::White, white_king_square)
            - king_tropism(position, Color::Black, black_king_square)
    }

    fn position_shelter(position: &Position) -> ValueScore {
        let white_king_square =
            position.board.pieces_bb_color(Piece::King, Color::White).into_iter().next().unwrap();
        let black_king_square =
            position.board.pieces_bb_color(Piece::King, Color::Black).into_iter().next().unwrap();

        super::king_pawn_shelter(position, Color::White, white_king_square)
            - super::king_pawn_shelter(position, Color::Black, black_king_square)
    }

    #[test]
    fn tropism_smoke() {
        let start_position = Position::from_fen(START_FEN).unwrap();
        assert_eq!(position_tropism(&start_position), 0);
    }

    #[test]
    fn tropism_strong() {
        let position =
            Position::from_fen("r5k1/2qb1p1p/5QpB/ppbpr3/2pN4/2P3P1/PP3P1P/3RR1K1 b - - 1 21")
                .unwrap();
        assert!(position_tropism(&position) > 20);
    }

    #[test]
    fn shelter_smoke() {
        let position = Position::from_fen(START_FEN).unwrap();
        assert_eq!(position_shelter(&position), 0);
    }

    #[test]
    fn broken_shelter_soft() {
        let position =
            Position::from_fen("r2q1rk1/1p2bppp/p2p4/3Ppb2/6P1/PN2BP2/1PP4P/R2Q1RK1 b - - 0 15")
                .unwrap();

        assert!((-40..=-20).contains(&position_shelter(&position)));
    }

    #[test]
    fn broken_shelter_hard() {
        let position =
            Position::from_fen("r4r1k/1p2p1pp/p2p2b1/3P4/6P1/PNP1q1P1/1P3R2/R2Q2K1 w - - 1 22")
                .unwrap();

        assert!((-120..=-50).contains(&position_shelter(&position)));
    }

    #[test]
    fn ok_shelter() {
        let position =
            Position::from_fen("r2q1rk1/1p2bppp/p2p4/3Ppb2/8/PN2BP2/1PP3PP/R2Q1RK1 w - - 1 15")
                .unwrap();

        assert!((-10..=-2).contains(&position_shelter(&position)));
    }
}
