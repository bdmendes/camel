use crate::position::{color::Color, square::Square, Position};

use super::{Move, MoveFlag};

pub fn make_move(position: &Position, mov: Move) -> Position {
    let mut position = *position;

    let piece = position.piece_at(mov.from()).unwrap();
    let side_to_move = position.side_to_move();

    position.clear_square(mov.from());

    match mov.flag() {
        MoveFlag::Quiet | MoveFlag::Capture | MoveFlag::DoublePawnPush => {
            position.set_square(mov.to(), piece, side_to_move);
        }
        MoveFlag::EnpassantCapture => {
            position.set_square(mov.to(), piece, side_to_move);
            position.clear_square(position.ep_square().unwrap().shift(match side_to_move {
                Color::White => Square::SOUTH,
                Color::Black => Square::NORTH,
            }));
        }
        MoveFlag::KingsideCastle => todo!(),
        MoveFlag::QueensideCastle => todo!(),
        MoveFlag::KnightPromotion => todo!(),
        MoveFlag::BishopPromotion => todo!(),
        MoveFlag::RookPromotion => todo!(),
        MoveFlag::QueenPromotion => todo!(),
        MoveFlag::KnightPromotionCapture => todo!(),
        MoveFlag::BishopPromotionCapture => todo!(),
        MoveFlag::RookPromotionCapture => todo!(),
        MoveFlag::QueenPromotionCapture => todo!(),
    }

    // TODO: other updates...

    position
}
