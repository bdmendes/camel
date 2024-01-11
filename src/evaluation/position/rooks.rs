use crate::{
    evaluation::ValueScore,
    position::{bitboard::Bitboard, board::Piece, Color, Position},
};

const CONNECTED_ROOK_BONUS: ValueScore = 5;
const SEMI_OPEN_FILE_BONUS: ValueScore = 10;
const OPEN_FILE_BONUS: ValueScore = 15;

pub fn evaluate_rooks(position: &Position) -> ValueScore {
    let mut score = 0;

    for color in Color::list() {
        let rooks = position.board.pieces_bb_color(Piece::Rook, *color);

        if let Some(rook) = rooks.into_iter().next() {
            let occupancy = position.board.occupancy_bb_all();
            let our_pawns = position.board.pieces_bb_color(Piece::Pawn, *color);
            let their_pawns = position.board.pieces_bb_color(Piece::Pawn, color.opposite());
            let our_file = Bitboard::file_mask(rook.file());
            let our_rank = Bitboard::rank_mask(rook.rank());

            // Connected rooks on file
            let rooks_on_file = rooks & our_file;
            if rooks_on_file.count_ones() > 1 {
                let first_rook = rooks_on_file.into_iter().next().unwrap();
                let second_rook = rooks_on_file.into_iter().nth(1).unwrap();
                let range = Bitboard::file_range(first_rook, second_rook) & !rooks_on_file;
                if (occupancy & range).is_empty() {
                    score += CONNECTED_ROOK_BONUS * color.sign();
                }
            }

            // Connected rooks on rank
            let rooks_on_rank = rooks & our_rank;
            if rooks_on_rank.count_ones() > 1 {
                let first_rook = rooks_on_rank.into_iter().next().unwrap();
                let second_rook = rooks_on_rank.into_iter().nth(1).unwrap();
                let range = Bitboard::rank_range(first_rook, second_rook) & !rooks_on_rank;
                if (occupancy & range).is_empty() {
                    score += CONNECTED_ROOK_BONUS * color.sign();
                }
            }

            // Semi-open files
            if (our_file & our_pawns).is_empty() {
                if (our_file & their_pawns).is_empty() {
                    score += OPEN_FILE_BONUS * color.sign();
                } else {
                    score += SEMI_OPEN_FILE_BONUS * color.sign();
                }
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::position::rooks::{
            CONNECTED_ROOK_BONUS, OPEN_FILE_BONUS, SEMI_OPEN_FILE_BONUS,
        },
        position::Position,
    };

    #[test]
    fn rook_open_file_both_connected_black() {
        let position =
            Position::from_fen("3rr1k1/pp3ppp/n1p2n2/4N3/8/5N2/PP2RPPP/3R1K2 w - - 3 17").unwrap();
        let rooks_score = super::evaluate_rooks(&position);
        assert_eq!(rooks_score, OPEN_FILE_BONUS - OPEN_FILE_BONUS - CONNECTED_ROOK_BONUS);
    }

    #[test]
    fn rook_open_file_both_connected_both() {
        let position =
            Position::from_fen("3rr1k1/pp3ppp/n1p2n2/4N3/8/5N2/PP2RPPP/4RK2 b - - 4 17").unwrap();
        let rooks_score = super::evaluate_rooks(&position);
        assert_eq!(rooks_score, CONNECTED_ROOK_BONUS - CONNECTED_ROOK_BONUS);
    }

    #[test]
    fn rook_semi_connected_white_open_file_white() {
        let position =
            Position::from_fen("2kr2nr/pbppb3/1pn1pq1p/6p1/2P5/4BNNP/PPQ1BPP1/3R1RK1 b - - 1 19")
                .unwrap();
        let rooks_score = super::evaluate_rooks(&position);
        assert_eq!(rooks_score, SEMI_OPEN_FILE_BONUS + CONNECTED_ROOK_BONUS);
    }
}
