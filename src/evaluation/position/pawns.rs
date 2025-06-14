use crate::{
    evaluation::ValueScore,
    moves::gen::MoveDirection,
    position::{bitboard::Bitboard, board::Piece, Color, Position},
};

pub static mut DOUBLED_PAWNS_PENALTY: ValueScore = -10;
pub static mut PAWN_ISLAND_PENALTY: ValueScore = -10;
pub static mut PASSED_PAWN_BONUS: [ValueScore; 8] = [0, 8, 9, 14, 41, 98, 158, 0];

fn doubled_pawns(bb: Bitboard) -> u8 {
    (0..8).fold(0, |acc, file| {
        let file_bb = bb & Bitboard::file_mask(file);
        let count = file_bb.count_ones() as u8;
        if count > 1 {
            acc + count - 1
        } else {
            acc
        }
    })
}

fn pawn_islands(bb: Bitboard) -> u8 {
    let mut islands = 0;
    let mut on_empty_file = true;

    for file in 0..8 {
        let file_bb = bb & Bitboard::file_mask(file);
        if file_bb.is_empty() {
            on_empty_file = true;
        } else if on_empty_file {
            islands += 1;
            on_empty_file = false;
        }
    }

    islands
}

type RelativeRank = u8;

fn passed_pawns(us_direction: i8, us_bb: Bitboard, them_bb: Bitboard) -> [RelativeRank; 8] {
    let mut passed_pawns_ranks = [0; 8];

    for file in 0..8 {
        let our_pawns_on_file = us_bb & Bitboard::file_mask(file);
        let our_most_advanced_pawn = if us_direction > 0 {
            our_pawns_on_file.into_iter().next_back()
        } else {
            our_pawns_on_file.into_iter().next()
        };
        if let Some(our_most_advanced_pawn) = our_most_advanced_pawn {
            let challenging_pawns_file_mask = match file {
                0 => Bitboard::file_mask(1),
                7 => Bitboard::file_mask(6),
                _ => Bitboard::file_mask(file - 1) | Bitboard::file_mask(file + 1),
            } | Bitboard::file_mask(file);
            let challenging_pawns_rank_mask = if us_direction > 0 {
                Bitboard::ranks_mask_up(our_most_advanced_pawn.rank())
            } else {
                Bitboard::ranks_mask_down(our_most_advanced_pawn.rank())
            };
            let challenging_pawns_bb =
                them_bb & challenging_pawns_file_mask & challenging_pawns_rank_mask;

            if challenging_pawns_bb.is_empty() {
                passed_pawns_ranks[file as usize] = if us_direction > 0 {
                    our_most_advanced_pawn.rank()
                } else {
                    7 - our_most_advanced_pawn.rank()
                };
            }
        }
    }

    passed_pawns_ranks
}

pub fn evaluate_pawn_structure(position: &Position) -> ValueScore {
    let mut score = 0;

    let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);
    let black_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::Black);

    unsafe {
        score += doubled_pawns(white_pawns) as ValueScore * DOUBLED_PAWNS_PENALTY;
        score -= doubled_pawns(black_pawns) as ValueScore * DOUBLED_PAWNS_PENALTY;

        score += pawn_islands(white_pawns) as ValueScore * PAWN_ISLAND_PENALTY;
        score -= pawn_islands(black_pawns) as ValueScore * PAWN_ISLAND_PENALTY;

        score +=
            passed_pawns(MoveDirection::pawn_direction(Color::White), white_pawns, black_pawns)
                .iter()
                .fold(0, |acc, rank| acc + PASSED_PAWN_BONUS[*rank as usize]);
        score -=
            passed_pawns(MoveDirection::pawn_direction(Color::Black), black_pawns, white_pawns)
                .iter()
                .fold(0, |acc, rank| acc + PASSED_PAWN_BONUS[*rank as usize]);
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::position::pawns::passed_pawns,
        moves::gen::MoveDirection,
        position::{board::Piece, fen::FromFen, Color, Position},
    };

    #[test]
    fn doubled_pawns_1() {
        let position = Position::from_fen("8/8/8/P7/P4P2/8/PPPP1PP1/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::doubled_pawns(white_pawns), 3);
    }

    #[test]
    fn double_pawns_2() {
        let position = Position::from_fen("8/8/7P/8/2P5/5PP1/PP1PP3/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::doubled_pawns(white_pawns), 0);
    }

    #[test]
    fn pawn_islands_1() {
        let position = Position::from_fen("8/8/7P/8/2P5/5PP1/PP1PP3/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::pawn_islands(white_pawns), 1);
    }

    #[test]
    fn pawn_islands_2() {
        let position = Position::from_fen("8/8/8/8/2P5/5PP1/1P1PP3/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::pawn_islands(white_pawns), 1);
    }

    #[test]
    fn pawn_islands_3() {
        let position = Position::from_fen("8/8/8/8/2P5/5PP1/1P1P4/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::pawn_islands(white_pawns), 2);
    }

    #[test]
    fn pawn_islands_4() {
        let position = Position::from_fen("8/8/8/8/8/P4PP1/1P1P4/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::pawn_islands(white_pawns), 3);
    }

    #[test]
    fn pawn_islands_5() {
        let position = Position::from_fen("8/8/8/8/8/P2P3P/8/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);

        assert_eq!(super::pawn_islands(white_pawns), 3);
    }

    #[test]
    fn passed_pawns_1() {
        let position = Position::from_fen("8/1p6/8/1pPP4/5p2/7P/5P2/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);
        let black_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::Black);

        assert_eq!(
            passed_pawns(MoveDirection::pawn_direction(Color::White), white_pawns, black_pawns),
            [0, 0, 0, 4, 0, 0, 0, 2]
        );

        assert_eq!(
            passed_pawns(MoveDirection::pawn_direction(Color::Black), black_pawns, white_pawns),
            [0, 3, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn passed_pawns_2() {
        let position = Position::from_fen("8/8/8/1pPPp1P1/1p3pP1/7P/8/8 w - - 0 1").unwrap();
        let white_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::White);
        let black_pawns = position.board.pieces_bb_color(Piece::Pawn, Color::Black);

        assert_eq!(
            passed_pawns(MoveDirection::pawn_direction(Color::White), white_pawns, black_pawns),
            [0, 0, 4, 4, 0, 0, 4, 2]
        );

        assert_eq!(
            passed_pawns(MoveDirection::pawn_direction(Color::Black), black_pawns, white_pawns),
            [0, 4, 0, 0, 3, 4, 0, 0]
        );
    }
}
