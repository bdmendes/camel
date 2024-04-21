use crate::{
    evaluation::{Evaluable, ValueScore},
    moves::gen::square_attackers,
    position::{
        bitboard::Bitboard,
        board::{Board, Piece},
        square::Square,
        Color, Position,
    },
};

static ATTACKS_WEIGHT: [i32; 9] = [0, 10, 32, 45, 60, 68, 76, 82, 90];

fn king_attackers_bonus(board: &Board, square: Square, by_color: Color) -> (ValueScore, Bitboard) {
    let mut bonus = 0;
    let square_attackers = square_attackers::<false>(board, square, by_color);

    for square in square_attackers {
        let piece = board.piece_at(square).unwrap();
        if piece == Piece::Pawn {
            continue;
        }

        bonus += piece.value() as ValueScore / 10;
    }

    (bonus, square_attackers & !board.pieces_bb(Piece::Pawn))
}

fn attacking_king_zone_bonus(board: &Board, color: Color, king_square: Square) -> ValueScore {
    let squares_around_king = king_square.squares_around();
    let mut attacking_bb = Bitboard::new(0);
    let mut bonus = 0;

    for square in squares_around_king {
        let (square_bonus, square_attackers_bb) =
            king_attackers_bonus(board, square, color.opposite());
        bonus += square_bonus;
        attacking_bb |= square_attackers_bb;
    }

    (bonus as i32 * ATTACKS_WEIGHT[attacking_bb.count_ones() as usize] / 100) as ValueScore
}

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

    const SHELTER_PENALTY: ValueScore = -20;

    for file in file_min..=file_max {
        let our_pawns_on_file = our_pawns & Bitboard::file_mask(file);
        let most_advanced_pawn = match king_color {
            Color::White => our_pawns_on_file.into_iter().next(),
            Color::Black => our_pawns_on_file.into_iter().next_back(),
        };
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
            shelter += SHELTER_PENALTY * 2;
        }
    }

    shelter
}

pub fn evaluate_king_safety(position: &Position, midgame_ratio: u8) -> ValueScore {
    let white_king_square =
        position.board.pieces_bb_color(Piece::King, Color::White).into_iter().next().unwrap();
    let black_king_square =
        position.board.pieces_bb_color(Piece::King, Color::Black).into_iter().next().unwrap();

    let mut score = 0;

    score += king_pawn_shelter(position, Color::White, white_king_square);
    score -= king_pawn_shelter(position, Color::Black, black_king_square);

    score += attacking_king_zone_bonus(&position.board, Color::Black, black_king_square);
    score -= attacking_king_zone_bonus(&position.board, Color::White, white_king_square);

    (score as i32 * midgame_ratio as i32 / 255) as ValueScore
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::{position::king::ATTACKS_WEIGHT, Evaluable, ValueScore},
        position::{
            board::Piece,
            fen::{FromFen, START_FEN},
            square::Square,
            Color, Position,
        },
    };

    fn position_shelter(position: &Position) -> ValueScore {
        let white_king_square =
            position.board.pieces_bb_color(Piece::King, Color::White).into_iter().next().unwrap();
        let black_king_square =
            position.board.pieces_bb_color(Piece::King, Color::Black).into_iter().next().unwrap();

        super::king_pawn_shelter(position, Color::White, white_king_square)
            - super::king_pawn_shelter(position, Color::Black, black_king_square)
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

    #[test]
    fn attacking_zone_square() {
        let position = Position::from_fen(
            "r1bqkb1r/pppn1ppp/2np4/4p1NQ/2B1P3/8/PPPP1PPP/RNB1K2R w KQkq - 2 6",
        )
        .unwrap();

        let expected_bonus =
            (Piece::Queen.value() + Piece::Knight.value() + Piece::Bishop.value()) / 10;
        assert_eq!(
            super::king_attackers_bonus(&position.board, Square::F7, Color::White).0,
            expected_bonus
        );
    }

    #[test]
    fn attacking_zone_bonus() {
        let position = Position::from_fen(
            "r1bqkb1r/1pp2ppp/pnnp4/4p1BQ/2B1P3/3P1N2/PPP2PPP/RN2K2R b KQkq - 2 8",
        )
        .unwrap();

        let expected_bonus_around_f7 = (Piece::Queen.value() + Piece::Bishop.value()) / 10;
        let expected_bonus_around_e7 = Piece::Bishop.value() / 10;
        let expected_bonus_around_d8 = Piece::Bishop.value() / 10;

        assert_eq!(
            super::attacking_king_zone_bonus(
                &position.board,
                Color::Black,
                position.board.pieces_bb_color(Piece::King, Color::Black).next().unwrap()
            ) as i32,
            (expected_bonus_around_f7 + expected_bonus_around_e7 + expected_bonus_around_d8) as i32
                * ATTACKS_WEIGHT[3]
                / 100
        );
    }

    #[test]
    fn attacked_king_smoke() {
        let position = Position::from_fen(
            "r1bqkb1r/1pp2ppp/pnnp4/4p1BQ/2B1P3/3P1N2/PPP2PPP/RN2K2R b KQkq - 2 8",
        )
        .unwrap();

        assert!(
            super::evaluate_king_safety(&position, 255) > 0
                && super::evaluate_king_safety(&position, 255) < 100
        );
    }
}
