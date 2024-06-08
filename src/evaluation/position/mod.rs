use self::{
    bishops::evaluate_bishops, king::evaluate_king_safety, pawns::evaluate_pawn_structure,
    rooks::evaluate_rooks,
};
use super::{psqt::psqt_value, Evaluable, ValueScore};
use crate::{
    moves::gen::piece_attacks,
    position::{board::Piece, Color, Position},
};

pub mod bishops;
pub mod king;
pub mod pawns;
pub mod rooks;

pub const MAX_POSITIONAL_GAIN: ValueScore = 200;

pub static mut PAWN_MIDGAME_RATIO: ValueScore = 1;
pub static mut KNIGHT_MIDGAME_RATIO: ValueScore = 5;
pub static mut BISHOP_MIDGAME_RATIO: ValueScore = 9;
pub static mut ROOK_MIDGAME_RATIO: ValueScore = 20;
pub static mut QUEEN_MIDGAME_RATIO: ValueScore = 38;

fn midgame_ratio(position: &Position) -> u8 {
    Piece::list().iter().fold(0, |acc, piece| {
        acc.saturating_add(
            position.board.pieces_bb(*piece).count_ones() as u8
                * unsafe {
                    match *piece {
                        Piece::Pawn => PAWN_MIDGAME_RATIO,
                        Piece::Knight => KNIGHT_MIDGAME_RATIO,
                        Piece::Bishop => BISHOP_MIDGAME_RATIO,
                        Piece::Rook => ROOK_MIDGAME_RATIO,
                        Piece::Queen => QUEEN_MIDGAME_RATIO,
                        Piece::King => 0,
                    }
                } as u8,
        )
    })
}

fn mobility_bonus(piece: Piece) -> ValueScore {
    match piece {
        Piece::Pawn => 0,
        Piece::Bishop => 3,
        Piece::Knight | Piece::Rook => 2,
        Piece::Queen => 1,
        Piece::King => 0,
    }
}

fn insufficient_material(position: &Position) -> bool {
    let pieces_count = position.board.occupancy_bb_all().count_ones();

    if pieces_count > 4 {
        return false;
    }

    let knights_bb = position.board.pieces_bb(Piece::Knight);
    if knights_bb.count_ones() == 2 {
        return true;
    }

    let bishops_bb = position.board.pieces_bb(Piece::Bishop);
    if pieces_count == 3 && (knights_bb | bishops_bb).is_not_empty() {
        return true;
    }

    false
}

impl Evaluable for Position {
    fn value(&self) -> ValueScore {
        if insufficient_material(self) {
            return 0;
        }

        let midgame_ratio = midgame_ratio(self);
        let endgame_ratio = 255 - midgame_ratio;
        let occupancy = self.board.occupancy_bb_all();

        let base_score = Piece::list().iter().fold(0, |acc, piece| {
            let piece_value = piece.value();
            let piece_mobility_bonus = mobility_bonus(*piece);
            let pieces_bb = self.board.pieces_bb(*piece);

            acc + Color::list().iter().fold(0, |acc, color| {
                let bb = pieces_bb & self.board.occupancy_bb(*color);

                let material_score = bb.count_ones() as ValueScore * piece_value;
                let positional_score = bb.into_iter().fold(0, |acc, square| {
                    acc + psqt_value(*piece, square, *color, endgame_ratio)
                        + piece_mobility_bonus
                            * piece_attacks(*piece, square, occupancy, *color).count_ones()
                                as ValueScore
                });

                acc + (positional_score + material_score) * color.sign()
            })
        });

        let pawns_score = evaluate_pawn_structure(self);
        let king_score = evaluate_king_safety(self, midgame_ratio);
        let rooks_score = evaluate_rooks(self);
        let bishops_score = evaluate_bishops(self);

        base_score + pawns_score + king_score + rooks_score + bishops_score
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::Evaluable,
        position::{
            fen::{FromFen, START_FEN},
            Position,
        },
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
