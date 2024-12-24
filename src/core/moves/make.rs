use crate::core::{
    bitboard::Bitboard, castling_rights::CastlingSide, color::Color, piece::Piece, square::Square,
    Position,
};

use super::{Move, MoveFlag};

static COLOR_CASTLE_RANKS: [Bitboard; 2] = [Bitboard::rank_mask(0), Bitboard::rank_mask(7)];
static TO_SQUARE_KINGSIDE: [Square; 2] = [Square::G1, Square::G8];
static TO_SQUARE_QUEENSIDE: [Square; 2] = [Square::C1, Square::C8];

fn make_castle<const UPDATE_META: bool>(
    position: &mut Position,
    side_to_move: Color,
    castling_side: CastlingSide,
) {
    let ours = position.occupancy_bb(side_to_move);
    let rooks = position.pieces_bb(Piece::Rook) & ours & COLOR_CASTLE_RANKS[side_to_move as usize];
    let (rook, to_square) = match castling_side {
        CastlingSide::Kingside => (rooks.msb(), TO_SQUARE_KINGSIDE[side_to_move as usize]),
        CastlingSide::Queenside => (rooks.lsb(), TO_SQUARE_QUEENSIDE[side_to_move as usize]),
    };

    position.clear_square(rook.unwrap());
    position.set_square(to_square, Piece::King, side_to_move);
    position.set_square(
        match castling_side {
            CastlingSide::Kingside => to_square >> 1,
            CastlingSide::Queenside => to_square << 1,
        },
        Piece::Rook,
        side_to_move,
    );

    if UPDATE_META {
        position.set_castling_rights(position.castling_rights().removed_color(side_to_move));
    }
}

pub fn make_move<const UPDATE_META: bool>(position: &Position, mov: Move) -> Position {
    let mut position = *position;

    let piece = position.piece_at(mov.from()).unwrap();
    let side_to_move = position.side_to_move();

    position.clear_square_low::<UPDATE_META>(mov.from());

    match mov.flag() {
        MoveFlag::Quiet | MoveFlag::Capture
            if UPDATE_META
                && piece == Piece::King
                && position.castling_rights().has_color(side_to_move) =>
        {
            position.set_square(mov.to(), piece, side_to_move);
            position.set_castling_rights(position.castling_rights().removed_color(side_to_move));
        }
        MoveFlag::Quiet | MoveFlag::Capture
            if UPDATE_META
                && piece == Piece::Rook
                && position.castling_rights().has_color(side_to_move)
                && COLOR_CASTLE_RANKS[side_to_move as usize].is_set(mov.from()) =>
        {
            position.set_square(mov.to(), piece, side_to_move);
            let our_king = position.pieces_color_bb(Piece::King, side_to_move).lsb().unwrap();
            position.set_castling_rights(position.castling_rights().removed_side(
                side_to_move,
                if mov.from().file() > our_king.file() {
                    CastlingSide::Kingside
                } else {
                    CastlingSide::Queenside
                },
            ));
        }
        MoveFlag::Quiet | MoveFlag::DoublePawnPush => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), piece, side_to_move);
        }
        MoveFlag::Capture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), piece, side_to_move);
        }
        MoveFlag::EnpassantCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), piece, side_to_move);
            position.clear_square_low::<UPDATE_META>(match side_to_move {
                Color::White => position.ep_square().unwrap() >> 8,
                Color::Black => position.ep_square().unwrap() << 8,
            });
        }
        MoveFlag::KnightPromotion => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Knight, side_to_move);
        }
        MoveFlag::KnightPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Knight, side_to_move);
        }
        MoveFlag::BishopPromotion => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Bishop, side_to_move);
        }
        MoveFlag::BishopPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Bishop, side_to_move);
        }
        MoveFlag::RookPromotion => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Rook, side_to_move);
        }
        MoveFlag::RookPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Rook, side_to_move);
        }
        MoveFlag::QueenPromotion => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Queen, side_to_move);
        }
        MoveFlag::QueenPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Queen, side_to_move);
        }
        MoveFlag::KingsideCastle => {
            make_castle::<UPDATE_META>(&mut position, side_to_move, CastlingSide::Kingside);
        }
        MoveFlag::QueensideCastle => {
            make_castle::<UPDATE_META>(&mut position, side_to_move, CastlingSide::Queenside);
        }
    }

    if !UPDATE_META {
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
            Color::White => mov.to() >> 8,
            Color::Black => mov.to() << 8,
        });
    } else {
        position.clear_ep_square();
    };

    position.flip_side_to_move();

    position
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::core::{moves::make::make_move, MoveStage, Position};
    use std::str::FromStr;

    #[rstest]
    #[case(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "e1g1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQ1RK1 b - - 2 8"
    )]
    #[case(
        "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
        "h2b8",
        "rB2k2r/1b4bq/8/8/8/8/8/R3K2R b KQkq - 1 1"
    )]
    #[case(
        "rB2k2r/1b4bq/8/8/8/8/8/R3K2R b KQkq - 1 1",
        "a8b8",
        "1r2k2r/1b4bq/8/8/8/8/8/R3K2R w KQk - 0 2"
    )]
    #[case(
        "r3k2r/8/3Q4/8/8/8/8/R2qK2R w KQkq - 1 2",
        "e1d1",
        "r3k2r/8/3Q4/8/8/8/8/R2K3R b kq - 0 2"
    )]
    fn make(#[case] position: &str, #[case] mov: &str, #[case] expected: &str) {
        let position = Position::from_str(position).unwrap();
        let moves = position.moves(MoveStage::All);
        let mov = moves.iter().find(|m| m.to_string().as_str() == mov).unwrap();
        assert_eq!(make_move::<true>(&position, *mov).fen().as_str(), expected);
    }
}
