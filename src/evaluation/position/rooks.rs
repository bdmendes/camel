use crate::{
    core::{bitboard::Bitboard, color::Color, piece::Piece, Position},
    evaluation::ValueScore,
};

pub static mut SEMI_OPEN_FILE_BONUS: ValueScore = 19;
pub static mut OPEN_FILE_BONUS: ValueScore = 21;

pub fn evaluate_rooks(position: &Position) -> ValueScore {
    let mut score = 0;

    unsafe {
        for color in Color::list() {
            let rooks = position.pieces_color_bb(Piece::Rook, *color);

            if let Some(rook) = rooks.into_iter().next() {
                let our_pawns = position.pieces_color_bb(Piece::Pawn, *color);
                let their_pawns = position.pieces_color_bb(Piece::Pawn, color.flipped());
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
    use std::str::FromStr;

    use crate::{core::Position, evaluation::position::rooks::SEMI_OPEN_FILE_BONUS};

    #[test]
    fn rook_semi_connected_white_open_file_white() {
        let position =
            Position::from_str("2kr2nr/pbppb3/1pn1pq1p/6p1/2P5/4BNNP/PPQ1BPP1/3R1RK1 b - - 1 19")
                .unwrap();
        let rooks_score = super::evaluate_rooks(&position);
        assert_eq!(rooks_score, unsafe { SEMI_OPEN_FILE_BONUS });
    }
}
