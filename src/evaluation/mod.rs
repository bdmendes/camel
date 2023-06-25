use self::psqt::psqt_value;
use crate::{
    moves::Move,
    position::{
        board::{Piece, PIECES, PIECES_WITH_PAWN},
        Color, Position,
    },
};

mod psqt;

pub type ValueScore = i16;

pub enum Score {
    IsMated(Color),
    Value(ValueScore),
}

fn piece_value(piece: Piece) -> ValueScore {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}

fn piece_endgame_ratio(piece: Piece) -> u8 {
    match piece {
        Piece::Pawn => 4,
        Piece::Knight => 8,
        Piece::Bishop => 8,
        Piece::Rook => 16,
        Piece::Queen => 32,
        Piece::King => 0,
    }
}

pub fn evaluate_position(position: &Position) -> ValueScore {
    let mut score = 0;

    // Endgame ratio
    let mut endgame_ratio = u8::MAX;
    for piece in PIECES.iter() {
        let bb = position.board.pieces_bb(*piece);
        endgame_ratio =
            endgame_ratio.saturating_sub(bb.count_ones() as u8 * piece_endgame_ratio(*piece));
    }

    // PSQT and piece values
    for piece in PIECES_WITH_PAWN.iter() {
        let mut bb = position.board.pieces_bb(*piece);
        while let Some(square) = bb.pop_lsb() {
            let color = position.board.color_at(square).unwrap();
            score += piece_value(*piece) * color.sign();
            score += psqt_value(*piece, square, color, endgame_ratio);
        }
    }

    score
}

pub fn evaluate_move(position: &Position, mov: Move) -> ValueScore {
    let mut score = 0;

    if mov.flag().is_capture() {
        let captured_piece = position.board.piece_at(mov.to()).unwrap().0;
        score += piece_value(captured_piece) + 100;
    }

    if mov.flag().is_promotion() {
        let promoted_piece = mov.promotion_piece().unwrap();
        score += match promoted_piece {
            Piece::Queen => 900,
            _ => -300,
        };
    }

    let piece = position.board.piece_at(mov.from()).unwrap().0;
    score += psqt_value(piece, mov.to(), position.side_to_move, 0);
    score -= psqt_value(piece, mov.from(), position.side_to_move, 0);

    score
}
