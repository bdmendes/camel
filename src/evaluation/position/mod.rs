use self::{king::evaluate_king_safety, pawns::evaluate_pawn_structure, psqt::psqt_value};
use super::{Evaluable, ValueScore};
use crate::{
    moves::gen::piece_attacks,
    position::{board::Piece, Color, Position},
};

mod king;
mod pawns;
pub mod psqt;

pub const MAX_POSITIONAL_GAIN: ValueScore = 300;

pub fn midgame_ratio(position: &Position) -> u8 {
    Piece::list().iter().fold(0, |acc, piece| {
        acc.saturating_add(
            position.board.pieces_bb(*piece).count_ones() as u8
                * match *piece {
                    Piece::Pawn => 4,
                    Piece::Knight => 10,
                    Piece::Bishop => 10,
                    Piece::Rook => 16,
                    Piece::Queen => 30,
                    Piece::King => 0,
                },
        )
    })
}

impl Evaluable for Position {
    fn value(&self) -> ValueScore {
        // Insufficient material
        if self.board.occupancy_bb_all().count_ones() <= 4 {
            // Two knights vs king
            let knights_bb = self.board.pieces_bb(Piece::Knight);
            if knights_bb.count_ones() == 2 {
                return 0;
            }

            // Knight/bishop vs king
            let bishops_bb = self.board.pieces_bb(Piece::Bishop);
            if self.board.occupancy_bb_all().count_ones() == 3
                && (knights_bb | bishops_bb).is_not_empty()
            {
                return 0;
            }
        }

        let midgame_ratio = midgame_ratio(self);
        let endgame_ratio = 255 - midgame_ratio;

        let occupancy = self.board.occupancy_bb_all();
        let white_occupancy = self.board.occupancy_bb(Color::White);
        let black_occupancy = self.board.occupancy_bb(Color::Black);

        let base_score = Piece::list().iter().fold(0, |acc, piece| {
            let bb = self.board.pieces_bb(*piece);
            let white_pieces = bb & white_occupancy;
            let black_pieces = bb & black_occupancy;

            // Material score
            let material_bonus = piece.value()
                * (white_pieces.count_ones() as ValueScore
                    - black_pieces.count_ones() as ValueScore);

            // PSQT score
            let white_positional_bonus = white_pieces.into_iter().fold(0, |acc, square| {
                acc + psqt_value(*piece, square, Color::White, endgame_ratio)
            });
            let black_positional_bonus = black_pieces.into_iter().fold(0, |acc, square| {
                acc + psqt_value(*piece, square, Color::Black, endgame_ratio)
            });
            let positional_bonus = white_positional_bonus - black_positional_bonus;

            // Mobility score
            let mobility_bonus = if *piece == Piece::Pawn {
                0
            } else {
                let piece_mobility_bonus = match *piece {
                    Piece::Bishop => 4,
                    Piece::Rook | Piece::Knight => 3,
                    Piece::Queen => 2,
                    Piece::King => -1,
                    _ => unreachable!(),
                };
                let white_mobility_bonus = white_pieces.into_iter().fold(0, |acc, square| {
                    let attacks = piece_attacks(*piece, square, occupancy) & !white_occupancy;
                    acc + attacks.count_ones() as ValueScore * piece_mobility_bonus
                });
                let black_mobility_bonus = black_pieces.into_iter().fold(0, |acc, square| {
                    let attacks = piece_attacks(*piece, square, occupancy) & !black_occupancy;
                    acc + attacks.count_ones() as ValueScore * piece_mobility_bonus
                });
                white_mobility_bonus - black_mobility_bonus
            };

            acc + material_bonus + positional_bonus + mobility_bonus
        });

        base_score + evaluate_pawn_structure(self) + evaluate_king_safety(self, midgame_ratio)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::Evaluable,
        position::{fen::START_FEN, Position},
    };

    #[test]
    fn eval_starts_zero() {
        let position = Position::from_fen(START_FEN).unwrap();
        assert_eq!(position.value(), 0);
    }

    #[test]
    fn eval_passed_extra_pawn_midgame() {
        let position =
            Position::from_fen("3r3k/1p1qQ1pp/p2P1n2/2p5/7B/P7/1P3PPP/4R1K1 w - - 5 26").unwrap();
        let evaluation = position.value();
        assert!(evaluation > 100 && evaluation < 300);
    }

    #[test]
    fn eval_forces_king_cornering() {
        let king_at_center_position =
            Position::from_fen("8/8/8/3K4/8/4q3/k7/8 b - - 6 55").unwrap();
        let king_at_corner_position =
            Position::from_fen("8/1K6/8/2q5/8/1k6/8/8 w - - 11 58").unwrap();
        let king_at_center_evaluation = king_at_center_position.value();
        let king_at_corner_evaluation = king_at_corner_position.value();
        assert!(king_at_center_evaluation > king_at_corner_evaluation);
    }
}
