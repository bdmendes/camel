use crate::position::{
    Position, bitboard::Bitboard, castling_rights::CastlingSide, color::Color, piece::Piece, square::Square,
};

use super::{Move, MoveFlag, generate::pawns::pawn_attackers};

const COLOR_CASTLE_RANKS: [Bitboard; 2] = [Bitboard::rank_mask(0), Bitboard::rank_mask(7)];
const TO_KING_KINGSIDE: [Square; 2] = [Square::G1, Square::G8];
const TO_KING_QUEENSIDE: [Square; 2] = [Square::C1, Square::C8];
const TO_ROOK_KINGSIDE: [Square; 2] = [Square::F1, Square::F8];
const TO_ROOK_QUEENSIDE: [Square; 2] = [Square::D1, Square::D8];

static CANDIDATE_EP: [Square; 128] = {
    let mut arr = [Square::A1; 128];
    let mut idx = 8;
    while idx < 128 {
        let to = if idx < 64 { idx - 8 } else { (idx - 64) + 8 };
        arr[idx] = Square::from_unsafe(to as u8 % 64);
        idx += 1;
    }
    arr
};

fn make_castle<const UPDATE_META: bool>(position: &mut Position, side_to_move: Color, castling_side: CastlingSide) {
    let ours = position.occupancy_bb(side_to_move);
    let rooks = position.pieces_bb(Piece::Rook) & ours & COLOR_CASTLE_RANKS[side_to_move as usize];
    let (rook, to_king, to_rook) = match castling_side {
        CastlingSide::Kingside => (
            rooks.msb(),
            TO_KING_KINGSIDE[side_to_move as usize],
            TO_ROOK_KINGSIDE[side_to_move as usize],
        ),
        CastlingSide::Queenside => (
            rooks.lsb(),
            TO_KING_QUEENSIDE[side_to_move as usize],
            TO_ROOK_QUEENSIDE[side_to_move as usize],
        ),
    };

    position.clear_square(rook.unwrap());
    position.set_square_low::<UPDATE_META, false>(to_king, Piece::King, side_to_move);
    position.set_square_low::<UPDATE_META, false>(to_rook, Piece::Rook, side_to_move);

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
            if UPDATE_META && piece == Piece::King && position.castling_rights().has_color(side_to_move) =>
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
        MoveFlag::Capture
            if UPDATE_META
                && position.piece_at(mov.to()) == Some(Piece::Rook)
                && COLOR_CASTLE_RANKS[side_to_move.flipped() as usize].is_set(mov.to()) =>
        {
            position.set_square(mov.to(), piece, side_to_move);
            let their_king = position
                .pieces_color_bb(Piece::King, side_to_move.flipped())
                .lsb()
                .unwrap();
            position.set_castling_rights(position.castling_rights().removed_side(
                side_to_move.flipped(),
                if mov.to().file() > their_king.file() {
                    CastlingSide::Kingside
                } else {
                    CastlingSide::Queenside
                },
            ));
        }
        MoveFlag::Quiet | MoveFlag::DoublePawnPush => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), piece, side_to_move);
        }
        MoveFlag::Capture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), piece, side_to_move);
        }
        MoveFlag::EnpassantCapture => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), piece, side_to_move);
            position.clear_square_low::<UPDATE_META>(
                CANDIDATE_EP[side_to_move as usize * 64 + position.ep_square().unwrap() as usize],
            );
        }
        MoveFlag::KnightPromotion => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), Piece::Knight, side_to_move);
        }
        MoveFlag::KnightPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Knight, side_to_move);
        }
        MoveFlag::BishopPromotion => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), Piece::Bishop, side_to_move);
        }
        MoveFlag::BishopPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Bishop, side_to_move);
        }
        MoveFlag::RookPromotion => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), Piece::Rook, side_to_move);
        }
        MoveFlag::RookPromotionCapture => {
            position.set_square_low::<UPDATE_META, true>(mov.to(), Piece::Rook, side_to_move);
        }
        MoveFlag::QueenPromotion => {
            position.set_square_low::<UPDATE_META, false>(mov.to(), Piece::Queen, side_to_move);
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
        let candidate_ep = CANDIDATE_EP[side_to_move as usize * 64 + mov.to() as usize];
        if !pawn_attackers(&position, side_to_move.flipped(), candidate_ep).is_empty() {
            position.set_ep_square(candidate_ep);
        } else {
            position.clear_ep_square();
        }
    } else {
        position.clear_ep_square();
    };

    position.flip_side();

    position
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{moves::make::make_move, position::Position};
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
    #[case("r3k2r/8/3Q4/8/8/8/8/R2qK2R w KQkq - 1 2", "e1d1", "r3k2r/8/3Q4/8/8/8/8/R2K3R b kq - 0 2")]
    #[case(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "e2e4",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    )]
    #[case("2rk4/8/8/3b4/8/8/8/4K2R b K - 0 1", "d5h1", "2rk4/8/8/8/8/8/8/4K2b w - - 0 2")]
    fn make(#[case] position: &str, #[case] mov: &str, #[case] expected: &str) {
        let position = Position::from_str(position).unwrap();
        let mov = position.get_move_str(mov).unwrap();
        assert_eq!(make_move::<true>(&position, mov).fen().as_str(), expected);
    }
}
