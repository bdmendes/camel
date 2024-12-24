use crate::core::{
    bitboard::Bitboard,
    castling_rights::CastlingSide,
    moves::{Move, MoveFlag},
    piece::Piece,
    square::Square,
    MoveStage, Position,
};

use super::square_attackers;

static COLOR_CASTLE_RANKS: [Bitboard; 2] = [Bitboard::rank_mask(0), Bitboard::rank_mask(7)];
static FINAL_KING_SQUARES: [Square; 4] = [Square::G1, Square::C1, Square::G8, Square::C8];
static FLAG_FROM_SIDE: [MoveFlag; 2] = [MoveFlag::KingsideCastle, MoveFlag::QueensideCastle];

fn castle_side(position: &Position, side: CastlingSide, moves: &mut Vec<Move>) {
    let king = position.pieces_color_bb(Piece::King, position.side_to_move).lsb().unwrap();
    let rook = {
        let our_rooks = position.pieces_color_bb(Piece::Rook, position.side_to_move)
            & COLOR_CASTLE_RANKS[position.side_to_move as usize];
        match side {
            CastlingSide::Kingside => our_rooks.msb(),
            CastlingSide::Queenside => our_rooks.lsb(),
        }
    };

    // The main move generator already verifies check before and after the move.
    // We only need to check for empty range and if the king goes through check.
    if let Some(rook) = rook {
        let invalid_rook = match side {
            CastlingSide::Kingside => rook.file() < king.file(),
            CastlingSide::Queenside => rook.file() > king.file(),
        };
        if invalid_rook {
            return;
        }

        let king_rook_range = Bitboard::between(king, rook);
        if !(position.occupancy_bb_all() & king_rook_range).is_empty() {
            return;
        }

        let final_king_square =
            FINAL_KING_SQUARES[(position.side_to_move as usize * 2) + (side as usize)];
        if position.chess960 {
            // In chess960, the king and rook jump over each other,
            // so we must check each path manually.
            let final_king_range_including = Bitboard::between(
                king,
                match side {
                    CastlingSide::Kingside => final_king_square << 1,
                    CastlingSide::Queenside => final_king_square >> 1,
                },
            );
            let final_rook_range_including = Bitboard::between(
                rook,
                match side {
                    CastlingSide::Kingside => final_king_square >> 2,
                    CastlingSide::Queenside => final_king_square << 2,
                },
            );
            if !(position.occupancy_bb_all()
                & !Bitboard::from_square(king)
                & !Bitboard::from_square(rook)
                & (final_king_range_including | final_rook_range_including))
                .is_empty()
            {
                return;
            }
        }

        let king_final_range = Bitboard::between(king, final_king_square);
        for sq in king_final_range {
            if !square_attackers(position, sq, position.side_to_move.flipped()).is_empty() {
                return;
            }
        }

        moves.push(Move::new(
            king,
            if position.chess960 { rook } else { final_king_square },
            FLAG_FROM_SIDE[side as usize],
        ));
    }
}

pub fn castle_moves(position: &Position, stage: MoveStage, moves: &mut Vec<Move>) {
    if matches!(stage, MoveStage::CapturesAndPromotions) {
        return;
    }

    if position.castling_rights().has_side(position.side_to_move, CastlingSide::Kingside) {
        castle_side(position, CastlingSide::Kingside, moves);
    }

    if position.castling_rights().has_side(position.side_to_move, CastlingSide::Queenside) {
        castle_side(position, CastlingSide::Queenside, moves);
    }
}

#[cfg(test)]
mod tests {
    use super::castle_moves;
    use crate::core::{MoveStage, Position};
    use std::str::FromStr;

    fn assert_castle(position: &str, moves: &[&str]) {
        let position = Position::from_str(position).unwrap();
        let mut buf = vec![];
        castle_moves(&position, MoveStage::All, &mut buf);

        assert_eq!(buf.len(), moves.len());

        for m in buf {
            assert!(moves.contains(&m.to_string().as_str()));
        }
    }

    #[test]
    fn regular_kingside() {
        assert_castle(
            "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            &["e1g1"],
        );
    }

    #[test]
    fn regular_queenside() {
        assert_castle(
            "r1b1r1k1/1pppqppp/p1n2n2/1Bb1p1B1/4P3/2NP4/PPPQ1PPP/R3K1NR w KQ - 0 9",
            &["e1c1"],
        );
    }

    #[test]
    fn through_check() {
        assert_castle(
            "r1b1r1k1/2ppqpp1/p2b1n2/1p4p1/BP2P3/P1NP1N1P/2PQ2Pn/R3K2R w KQ - 1 16",
            &["e1c1"],
        );
    }

    #[test]
    fn chess960_queenside() {
        assert_castle(
            "rbnkr1bq/pp2p2p/2p1n1p1/3p1p2/5P2/P2N2P1/BPPPP2P/R2KRNBQ w KQkq - 0 6",
            &["d1a1"],
        );
    }

    #[test]
    fn chess960_kingside() {
        assert_castle(
            "rb1kr2q/pp1ppbpp/2pnn3/5p2/5P2/P2NNQP1/1PPPP2P/RB1KR1B1 b KQkq - 4 6",
            &["d8e8"],
        );
    }

    #[test]
    fn single_rook() {
        assert_castle(
            "r1bqk2r/ppppbppp/2n1p3/7n/3PPB2/P1N2N1P/1PPQB1P1/R3K3 w Qkq - 1 10",
            &["e1c1"],
        );
    }

    #[test]
    fn rook_attacked_kingside() {
        assert_castle("r2qk1nr/pbppbppp/np2p3/4P3/8/6PB/PPPPNP1P/RNBQK2R w KQkq - 4 6", &["e1g1"]);
    }

    #[test]
    fn rook_attacked_queenside() {
        assert_castle("r3kbnr/p1pnqppp/1pBpp3/4P3/8/6P1/PPPPNP1P/RNBQK2R b KQkq - 2 7", &["e8c8"]);
    }

    #[test]
    fn rook_through_attack() {
        assert_castle(
            "r3kbnr/p1pnqppp/NpBpp3/4P3/8/6P1/PPPPNP1P/R1BQK2R b KQkq - 10 11",
            &["e8c8"],
        );
    }
}
