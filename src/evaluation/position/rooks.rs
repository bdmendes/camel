use crate::{
    evaluation::ValueScore,
    position::{bitboard::Bitboard, board::Piece, Color, Position},
};

pub static mut SEMI_OPEN_FILE_BONUS: ValueScore = 19;
pub static mut OPEN_FILE_BONUS: ValueScore = 21;

pub fn evaluate_rooks(position: &Position) -> ValueScore {
    let mut score = 0;

    unsafe {
        for color in Color::list() {
            let rooks = position.board.pieces_bb_color(Piece::Rook, *color);

            if let Some(rook) = rooks.into_iter().next() {
                let our_pawns = position.board.pieces_bb_color(Piece::Pawn, *color);
                let their_pawns = position.board.pieces_bb_color(Piece::Pawn, color.opposite());
                let our_file = Bitboard::file_mask(rook.file());

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
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::position::rooks::SEMI_OPEN_FILE_BONUS,
        position::{fen::FromFen, Position},
    };

    #[test]
    fn rook_semi_connected_white_open_file_white() {
        let position =
            Position::from_fen("2kr2nr/pbppb3/1pn1pq1p/6p1/2P5/4BNNP/PPQ1BPP1/3R1RK1 b - - 1 19")
                .unwrap();
        let rooks_score = super::evaluate_rooks(&position);
        assert_eq!(rooks_score, unsafe { SEMI_OPEN_FILE_BONUS });
    }
}
