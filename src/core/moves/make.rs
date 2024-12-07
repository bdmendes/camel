use crate::core::{
    bitboard::Bitboard, castling_rights::CastlingSide, color::Color, piece::Piece, square::Square,
    Position,
};

use super::{Move, MoveFlag};

static COLOR_CASTLE_RANKS: [Bitboard; 2] = [Bitboard::rank_mask(0), Bitboard::rank_mask(7)];

fn make_castle<const UPDATE_METADATA: bool>(
    position: &mut Position,
    side_to_move: Color,
    castling_side: CastlingSide,
    to_square: Square,
) {
    let ours = position.occupancy_bb(side_to_move);
    let mut rooks =
        position.pieces_bb(Piece::Rook) & ours & COLOR_CASTLE_RANKS[side_to_move as usize];
    let rook = match castling_side {
        CastlingSide::Kingside => rooks.next_back(),
        CastlingSide::Queenside => rooks.next(),
    };

    position.clear_square(rook.unwrap());
    position.set_square(to_square, Piece::King, side_to_move);
    position.set_square(
        to_square.shifted(match castling_side {
            CastlingSide::Kingside => Square::WEST,
            CastlingSide::Queenside => Square::EAST,
        }),
        Piece::Rook,
        side_to_move,
    );

    if UPDATE_METADATA {
        position.set_castling_rights(position.castling_rights().removed_color(side_to_move));
    }
}

pub fn make_move<const UPDATE_METADATA: bool>(position: &Position, mov: Move) -> Position {
    let mut position = *position;

    let piece = position.piece_at(mov.from()).unwrap();
    let side_to_move = position.side_to_move();

    position.clear_square(mov.from());

    match mov.flag() {
        MoveFlag::Quiet
            if UPDATE_METADATA && position.pieces_bb(Piece::King).is_set(mov.from()) =>
        {
            position.set_square(mov.to(), piece, side_to_move);
            position.set_castling_rights(position.castling_rights().removed_color(side_to_move));
        }
        MoveFlag::Quiet
            if UPDATE_METADATA
                && (position.pieces_bb(Piece::Rook)
                    & COLOR_CASTLE_RANKS[side_to_move as usize])
                    .is_set(mov.from()) =>
        {
            position.set_square(mov.to(), piece, side_to_move);
            let our_king = (position.pieces_bb(Piece::King) & position.occupancy_bb(side_to_move))
                .next()
                .unwrap();
            position.set_castling_rights(position.castling_rights().removed_side(
                side_to_move,
                if mov.from().file() > our_king.file() {
                    CastlingSide::Kingside
                } else {
                    CastlingSide::Queenside
                },
            ));
        }
        MoveFlag::Quiet | MoveFlag::Capture | MoveFlag::DoublePawnPush => {
            position.set_square(mov.to(), piece, side_to_move);
        }
        MoveFlag::EnpassantCapture => {
            position.set_square(mov.to(), piece, side_to_move);
            if UPDATE_METADATA {
                position.clear_square(position.ep_square().unwrap().shifted(match side_to_move {
                    Color::White => Square::SOUTH,
                    Color::Black => Square::NORTH,
                }));
            }
        }
        MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => {
            position.set_square(mov.to(), Piece::Knight, side_to_move);
        }
        MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => {
            position.set_square(mov.to(), Piece::Bishop, side_to_move);
        }
        MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => {
            position.set_square(mov.to(), Piece::Rook, side_to_move);
        }
        MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => {
            position.set_square(mov.to(), Piece::Queen, side_to_move);
        }
        MoveFlag::KingsideCastle => {
            make_castle::<UPDATE_METADATA>(
                &mut position,
                side_to_move,
                CastlingSide::Kingside,
                mov.to(),
            );
        }
        MoveFlag::QueensideCastle => {
            make_castle::<UPDATE_METADATA>(
                &mut position,
                side_to_move,
                CastlingSide::Queenside,
                mov.to(),
            );
        }
    }

    if !UPDATE_METADATA {
        return position;
    }

    if matches!(side_to_move, Color::Black) {
        position.set_fullmove_number(position.fullmove_number().saturating_add(1));
    }

    position.set_halfmove_clock(if mov.is_capture() || piece == Piece::Pawn {
        0
    } else {
        position.halfmove_clock().saturating_add(1)
    });

    if mov.flag() == MoveFlag::DoublePawnPush {
        position.set_ep_square(match side_to_move {
            Color::White => mov.to().shifted(Square::SOUTH),
            Color::Black => mov.to().shifted(Square::NORTH),
        });
    } else {
        position.clear_ep_square();
    };

    position.flip_side_to_move();

    position
}
