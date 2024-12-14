use crate::core::{
    bitboard::Bitboard,
    castling_rights::CastlingSide,
    color::Color,
    moves::{Move, MoveFlag},
    piece::Piece,
    square::Square,
    MoveStage, Position,
};

use super::square_attackers;

static COLOR_CASTLE_RANKS: [Bitboard; 2] = [Bitboard::rank_mask(0), Bitboard::rank_mask(7)];
static COLOR_KINGSIDE_SQUARES: [Square; 2] = [Square::G1, Square::G8];
static COLOR_QUEENSIDE_SQUARES: [Square; 2] = [Square::C1, Square::C8];

fn king_square(position: &Position) -> Square {
    position
        .pieces_color_bb(Piece::King, position.side_to_move)
        .next()
        .unwrap()
}

fn kingside_castle(position: &Position, moves: &mut Vec<Move>) {
    let king = king_square(position);
    let our_rank = COLOR_CASTLE_RANKS[position.side_to_move as usize];
    let our_rook =
        (our_rank & position.pieces_color_bb(Piece::Rook, position.side_to_move)).next_back();

    if let Some(rook) = our_rook {
        if rook.file() < king.file() {
            return;
        }

        let until_rook = Bitboard::between(king, rook);
        if !(position.occupancy_bb_all() & until_rook).is_empty() {
            return;
        }

        let until_final_king = Bitboard::between(
            king,
            COLOR_KINGSIDE_SQUARES[position.side_to_move as usize] << 1,
        );
        for sq in until_final_king {
            if !square_attackers(position, sq, position.side_to_move.flipped()).is_empty() {
                return;
            }
        }

        moves.push(Move::new(
            king,
            COLOR_KINGSIDE_SQUARES[position.side_to_move as usize],
            MoveFlag::KingsideCastle,
        ));
    }
}

fn queenside_castle(position: &Position, moves: &mut Vec<Move>) {
    let king = king_square(position);
    let our_rank = COLOR_CASTLE_RANKS[position.side_to_move as usize];
    let our_rook = (our_rank & position.pieces_color_bb(Piece::Rook, position.side_to_move)).next();

    if let Some(rook) = our_rook {
        if rook.file() > king.file() {
            return;
        }

        let until_rook = Bitboard::between(king, rook);
        if !(position.occupancy_bb_all() & until_rook).is_empty() {
            return;
        }

        let until_final_king = Bitboard::between(
            king,
            COLOR_QUEENSIDE_SQUARES[position.side_to_move as usize] >> 1,
        );
        for sq in until_final_king {
            if !square_attackers(position, sq, position.side_to_move.flipped()).is_empty() {
                return;
            }
        }

        moves.push(Move::new(
            king,
            COLOR_QUEENSIDE_SQUARES[position.side_to_move as usize],
            MoveFlag::QueensideCastle,
        ));
    }
}

pub fn castle_moves(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    if matches!(stage, MoveStage::CapturesAndPromotions)
        || !position.castling_rights().has_color(position.side_to_move)
        || position.is_check()
    {
        return;
    }

    if position
        .castling_rights()
        .has_side(position.side_to_move, CastlingSide::Kingside)
    {
        kingside_castle(position, moves);
    }

    if position
        .castling_rights()
        .has_side(position.side_to_move, CastlingSide::Queenside)
    {
        queenside_castle(position, moves);
    }
}
