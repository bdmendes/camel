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
static REGULAR_CHESS_ROOKS: Bitboard = Bitboard::new(
    (1 << Square::A1 as usize)
        | (1 << Square::A8 as usize)
        | (1 << Square::H1 as usize)
        | (1 << Square::H8 as usize),
);
static REGULAR_CHESS_KINGS: [Square; 2] = [Square::E1, Square::E8];

fn is_chess960(king: Square, rook: Square, side_to_move: Color) -> bool {
    !REGULAR_CHESS_ROOKS.is_set(rook) || REGULAR_CHESS_KINGS[side_to_move as usize] != king
}

fn king_square(position: &Position) -> Square {
    position
        .pieces_color_bb(Piece::King, position.side_to_move)
        .next()
        .unwrap()
}

fn castle_side<const QUEENSIDE: bool>(position: &Position, moves: &mut Vec<Move>) {
    let king = king_square(position);
    let our_rank = COLOR_CASTLE_RANKS[position.side_to_move as usize];
    let mut our_rook = (our_rank & position.pieces_color_bb(Piece::Rook, position.side_to_move));
    let our_rook = if QUEENSIDE {
        our_rook.next()
    } else {
        our_rook.next_back()
    };
    let castle_squares = if QUEENSIDE {
        &COLOR_QUEENSIDE_SQUARES
    } else {
        &COLOR_KINGSIDE_SQUARES
    };

    if let Some(rook) = our_rook {
        if (QUEENSIDE && rook.file() > king.file()) || (!QUEENSIDE && rook.file() < king.file()) {
            return;
        }

        let until_rook = Bitboard::between(king, rook);
        if !(position.occupancy_bb_all() & until_rook).is_empty() {
            return;
        }

        let until_final_king = Bitboard::between(
            king,
            if QUEENSIDE {
                castle_squares[position.side_to_move as usize] >> 1
            } else {
                castle_squares[position.side_to_move as usize] << 1
            },
        );
        if !(position.occupancy_bb_all() & until_final_king & !Bitboard::from_square(rook))
            .is_empty()
        {
            return;
        }
        for sq in until_final_king {
            if !square_attackers(position, sq, position.side_to_move.flipped()).is_empty() {
                return;
            }
        }

        moves.push(Move::new(
            king,
            if is_chess960(king, rook, position.side_to_move) {
                rook
            } else {
                castle_squares[position.side_to_move as usize]
            },
            if QUEENSIDE {
                MoveFlag::QueensideCastle
            } else {
                MoveFlag::KingsideCastle
            },
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
        castle_side::<false>(position, moves);
    }

    if position
        .castling_rights()
        .has_side(position.side_to_move, CastlingSide::Queenside)
    {
        castle_side::<true>(position, moves);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::core::{MoveStage, Position};

    use super::castle_moves;

    fn assert_castle(position: &str, moves: &[&str]) {
        let position = Position::from_str(position).unwrap();
        let mut buf = vec![];
        castle_moves(&position, MoveStage::All, &mut buf);

        for m in &buf {
            println!("{m}");
        }

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
            "r1b1r1k1/2ppqpp1/p4n2/1pb1n1p1/B3P3/2NP1N2/PPPQ2PP/R3K2R w KQ - 0 13",
            &["e1c1"],
        );
    }

    #[test]
    fn through_check_2() {
        assert_castle(
            "r3kb1r/pBpnqppp/Np1pp2n/4P3/8/6P1/PPPPNP1P/R1BQK2R b KQkq - 12 12",
            &[],
        );
    }

    #[test]
    fn in_check() {
        assert_castle(
            "r1b1kbnr/pp3ppp/2n5/qB1pP3/8/5N2/PPP2PPP/RNBQK2R w KQkq - 3 7",
            &[],
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
            "r1bqk2r/ppppbppp/2n1p3/7n/3PPB2/P1N2N1P/1PPQB1P1/R3K3 w KQkq - 1 10",
            &["e1c1"],
        );
    }

    #[test]
    fn rook_attacked_kingside() {
        assert_castle(
            "r2qk1nr/pbppbppp/np2p3/4P3/8/6PB/PPPPNP1P/RNBQK2R w KQkq - 4 6",
            &["e1g1"],
        );
    }

    #[test]
    fn rook_attacked_queenside() {
        assert_castle(
            "r3kbnr/p1pnqppp/1pBpp3/4P3/8/6P1/PPPPNP1P/RNBQK2R b KQkq - 2 7",
            &["e8c8"],
        );
    }

    #[test]
    fn rook_through_attack() {
        assert_castle(
            "r3kbnr/p1pnqppp/NpBpp3/4P3/8/6P1/PPPPNP1P/R1BQK2R b KQkq - 10 11",
            &["e8c8"],
        );
    }
}
