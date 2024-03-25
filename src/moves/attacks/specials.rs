use crate::{
    moves::{
        gen::{square_attackers, MoveDirection, MoveStage},
        Move, MoveFlag,
    },
    position::{
        bitboard::Bitboard,
        board::{Board, Piece},
        square::Square,
        CastlingRights, Color, Position,
    },
};

const PAWN_WEST_EDGE_FILE: Bitboard = Bitboard::file_mask(0);
const PAWN_EAST_EDGE_FILE: Bitboard = Bitboard::file_mask(7);
const PAWN_PROMOTION_RANKS: Bitboard =
    Bitboard::new(Bitboard::rank_mask(0).raw() | Bitboard::rank_mask(7).raw());
const THIRD_ROW_WHITE: Bitboard = Bitboard::rank_mask(2);
const THIRD_ROW_BLACK: Bitboard = Bitboard::rank_mask(5);
const FIRST_ROW_WHITE: Bitboard = Bitboard::rank_mask(0);
const FIRST_ROW_BLACK: Bitboard = Bitboard::rank_mask(7);

pub fn pawn_attacks(board: &Board, color: Color) -> Bitboard {
    let our_pawns = board.pieces_bb_color(Piece::Pawn, color);
    let direction = MoveDirection::pawn_direction(color);

    (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST)
        | (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST)
}

pub fn generate_pawn_moves(stage: MoveStage, position: &Position, moves: &mut Vec<Move>) {
    let occupancy = position.board.occupancy_bb_all();
    let occupancy_them = position.board.occupancy_bb(position.side_to_move.opposite());
    let our_pawns = position.board.pieces_bb_color(Piece::Pawn, position.side_to_move);

    let direction = MoveDirection::pawn_direction(position.side_to_move);

    // Single push
    let single_push_pawns = our_pawns.shift(direction) & !occupancy;
    let single_push_pawns_on_stage = match stage {
        MoveStage::HashMove => panic!("Hash move should not be generated here"),
        MoveStage::CapturesAndPromotions => single_push_pawns & PAWN_PROMOTION_RANKS,
        MoveStage::NonCaptures => single_push_pawns & !PAWN_PROMOTION_RANKS,
        MoveStage::All => single_push_pawns,
    };
    for to_square in single_push_pawns_on_stage {
        let from_square = to_square.shift(-direction).unwrap();
        push_pawn_move(occupancy, moves, from_square, to_square);
    }

    if matches!(stage, MoveStage::All | MoveStage::NonCaptures) {
        // Double push
        let third_row_bb = match position.side_to_move {
            Color::White => THIRD_ROW_WHITE,
            Color::Black => THIRD_ROW_BLACK,
        };
        let double_push_pawns = (single_push_pawns & third_row_bb).shift(direction) & !occupancy;

        for to_square in double_push_pawns {
            let from_square = to_square.shift(-direction * 2).unwrap();
            moves.push(Move::new(from_square, to_square, MoveFlag::DoublePawnPush));
        }
    }

    if matches!(stage, MoveStage::All | MoveStage::CapturesAndPromotions) {
        // West capture
        let west_pawns = (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST)
            & occupancy_them;
        for to_square in west_pawns {
            let from_square = to_square.shift(-direction - MoveDirection::WEST).unwrap();
            push_pawn_move(occupancy_them, moves, from_square, to_square);
        }

        // East capture
        let east_pawns = (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST)
            & occupancy_them;
        for to_square in east_pawns {
            let from_square = to_square.shift(-direction - MoveDirection::EAST).unwrap();
            push_pawn_move(occupancy_them, moves, from_square, to_square);
        }

        // En passant
        if let Some(en_passant_square) = position.en_passant_square {
            let ep_bb = Bitboard::new(1 << en_passant_square as u8);

            let west_pawn =
                (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST) & ep_bb;
            if let Some(to_square) = west_pawn.into_iter().next() {
                let from_square = to_square.shift(-direction - MoveDirection::WEST).unwrap();
                moves.push(Move::new(from_square, to_square, MoveFlag::EnPassantCapture));
            }

            let east_pawns =
                (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST) & ep_bb;
            if let Some(to_square) = east_pawns.into_iter().next() {
                let from_square = to_square.shift(-direction - MoveDirection::EAST).unwrap();
                moves.push(Move::new(from_square, to_square, MoveFlag::EnPassantCapture));
            }
        }
    }
}

fn push_pawn_move(
    occupancy_them: Bitboard,
    moves: &mut Vec<Move>,
    from_square: Square,
    to_square: Square,
) {
    let is_promotion = to_square.rank() == 0 || to_square.rank() == 7;

    if is_promotion {
        push_pawn_promotion(occupancy_them, moves, from_square, to_square);
    } else {
        let is_capture = occupancy_them.is_set(to_square);
        moves.push(Move::new(
            from_square,
            to_square,
            if is_capture { MoveFlag::Capture } else { MoveFlag::Quiet },
        ));
    }
}

fn push_pawn_promotion(
    occupancy: Bitboard,
    moves: &mut Vec<Move>,
    from_square: Square,
    to_square: Square,
) {
    let is_capture = occupancy.is_set(to_square);

    moves.push(Move::new(
        from_square,
        to_square,
        if is_capture { MoveFlag::QueenPromotionCapture } else { MoveFlag::QueenPromotion },
    ));

    moves.push(Move::new(
        from_square,
        to_square,
        if is_capture { MoveFlag::RookPromotionCapture } else { MoveFlag::RookPromotion },
    ));

    moves.push(Move::new(
        from_square,
        to_square,
        if is_capture { MoveFlag::BishopPromotionCapture } else { MoveFlag::BishopPromotion },
    ));

    moves.push(Move::new(
        from_square,
        to_square,
        if is_capture { MoveFlag::KnightPromotionCapture } else { MoveFlag::KnightPromotion },
    ));
}

pub fn generate_king_castles(position: &Position, moves: &mut Vec<Move>) {
    match position.side_to_move {
        Color::White => {
            if position.castling_rights.contains(CastlingRights::WHITE_KINGSIDE) {
                generate_kingside_castle(Color::White, position, moves);
            }
            if position.castling_rights.contains(CastlingRights::WHITE_QUEENSIDE) {
                generate_queenside_castle(Color::White, position, moves);
            }
        }
        Color::Black => {
            if position.castling_rights.contains(CastlingRights::BLACK_KINGSIDE) {
                generate_kingside_castle(Color::Black, position, moves);
            }
            if position.castling_rights.contains(CastlingRights::BLACK_QUEENSIDE) {
                generate_queenside_castle(Color::Black, position, moves);
            }
        }
    }
}

fn generate_kingside_castle(color: Color, position: &Position, moves: &mut Vec<Move>) {
    let rooks = position.board.pieces_bb_color(Piece::Rook, color);
    let king_square = (position.board.pieces_bb_color(Piece::King, color)).next();
    let right_hand_side_rook_square = (match color {
        Color::White => FIRST_ROW_WHITE,
        Color::Black => FIRST_ROW_BLACK,
    } & rooks)
        .into_iter()
        .next_back();

    let may_castle = right_hand_side_rook_square.is_some()
        && right_hand_side_rook_square.unwrap().file() > king_square.unwrap().file()
        && castle_range_ok(
            color,
            position.board,
            king_square.unwrap(),
            right_hand_side_rook_square.unwrap(),
        );
    if may_castle {
        moves.push(Move::new(
            king_square.unwrap(),
            position
                .is_chess960
                .then(|| right_hand_side_rook_square.unwrap())
                .unwrap_or_else(|| king_square.unwrap().shift(MoveDirection::EAST * 2).unwrap()),
            MoveFlag::KingsideCastle,
        ));
    }
}

fn generate_queenside_castle(color: Color, position: &Position, moves: &mut Vec<Move>) {
    let rooks = position.board.pieces_bb_color(Piece::Rook, color);
    let king_square = (position.board.pieces_bb_color(Piece::King, color)).next();
    let left_hand_side_rook_square = (match color {
        Color::White => FIRST_ROW_WHITE,
        Color::Black => FIRST_ROW_BLACK,
    } & rooks)
        .into_iter()
        .next();

    let may_castle = left_hand_side_rook_square.is_some()
        && left_hand_side_rook_square.unwrap().file() < king_square.unwrap().file()
        && castle_range_ok(
            color,
            position.board,
            king_square.unwrap(),
            left_hand_side_rook_square.unwrap(),
        );
    if may_castle {
        moves.push(Move::new(
            king_square.unwrap(),
            position
                .is_chess960
                .then(|| left_hand_side_rook_square.unwrap())
                .unwrap_or_else(|| king_square.unwrap().shift(MoveDirection::WEST * 2).unwrap()),
            MoveFlag::QueensideCastle,
        ));
    }
}

fn castle_range_ok(color: Color, board: Board, king_square: Square, rook_square: Square) -> bool {
    let final_king_square = if rook_square.file() > king_square.file() {
        match color {
            Color::White => Square::G1,
            Color::Black => Square::G8,
        }
    } else {
        match color {
            Color::White => Square::C1,
            Color::Black => Square::C8,
        }
    };
    let final_rook_square = if rook_square.file() > king_square.file() {
        match color {
            Color::White => Square::F1,
            Color::Black => Square::F8,
        }
    } else {
        match color {
            Color::White => Square::D1,
            Color::Black => Square::D8,
        }
    };

    let occupied_range = Bitboard::rank_range(king_square, rook_square)
        | Bitboard::rank_range(king_square, final_king_square)
        | Bitboard::rank_range(rook_square, final_rook_square);

    if !king_rook_range_occupied_ok(occupied_range, color, board) {
        return false;
    }

    let mut attacked_range = Bitboard::rank_range(king_square, final_king_square);

    attacked_range.all(|sq| square_attackers::<true>(&board, sq, color.opposite()).is_empty())
}

fn king_rook_range_occupied_ok(range: Bitboard, own_color: Color, board: Board) -> bool {
    let mut found_own_king = false;
    let mut found_own_rook = false;

    for square in range {
        let color = board.color_at(square);
        if let Some(color) = color {
            if color != own_color {
                return false;
            }
            if board.pieces_bb(Piece::King).is_set(square) {
                if found_own_king {
                    return false;
                }
                found_own_king = true;
            } else if board.pieces_bb(Piece::Rook).is_set(square) {
                if found_own_rook {
                    return false;
                }
                found_own_rook = true;
            } else {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::generate_king_castles;
    use crate::{
        moves::{attacks::specials::generate_pawn_moves, gen::MoveStage, Move, MoveFlag},
        position::{fen::FromFen, square::Square, Position},
    };

    #[test]
    fn pawn_moves_white() {
        let position =
            Position::from_fen("4r3/3P3p/8/pp3pp1/PP1pP1PP/8/2p2P2/1B6 w - - 0 1").unwrap();

        let expected_moves = &[
            Move::new(Square::A4, Square::B5, MoveFlag::Capture),
            Move::new(Square::B4, Square::A5, MoveFlag::Capture),
            Move::new(Square::D7, Square::D8, MoveFlag::QueenPromotion),
            Move::new(Square::D7, Square::D8, MoveFlag::RookPromotion),
            Move::new(Square::D7, Square::D8, MoveFlag::BishopPromotion),
            Move::new(Square::D7, Square::D8, MoveFlag::KnightPromotion),
            Move::new(Square::D7, Square::E8, MoveFlag::QueenPromotionCapture),
            Move::new(Square::D7, Square::E8, MoveFlag::RookPromotionCapture),
            Move::new(Square::D7, Square::E8, MoveFlag::BishopPromotionCapture),
            Move::new(Square::D7, Square::E8, MoveFlag::KnightPromotionCapture),
            Move::new(Square::E4, Square::E5, MoveFlag::Quiet),
            Move::new(Square::E4, Square::F5, MoveFlag::Capture),
            Move::new(Square::F2, Square::F3, MoveFlag::Quiet),
            Move::new(Square::F2, Square::F4, MoveFlag::DoublePawnPush),
            Move::new(Square::G4, Square::F5, MoveFlag::Capture),
            Move::new(Square::H4, Square::G5, MoveFlag::Capture),
            Move::new(Square::H4, Square::H5, MoveFlag::Quiet),
        ];

        let mut moves = Vec::new();
        generate_pawn_moves(MoveStage::All, &position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn pawn_moves_black() {
        let position =
            Position::from_fen("4r3/3P3p/8/pp3pp1/PP1pP1PP/8/2p2P2/1B6 b - e3 0 1").unwrap();

        let expected_moves = &[
            Move::new(Square::H7, Square::H6, MoveFlag::Quiet),
            Move::new(Square::H7, Square::H5, MoveFlag::DoublePawnPush),
            Move::new(Square::G5, Square::H4, MoveFlag::Capture),
            Move::new(Square::F5, Square::G4, MoveFlag::Capture),
            Move::new(Square::F5, Square::F4, MoveFlag::Quiet),
            Move::new(Square::F5, Square::E4, MoveFlag::Capture),
            Move::new(Square::D4, Square::D3, MoveFlag::Quiet),
            Move::new(Square::D4, Square::E3, MoveFlag::EnPassantCapture),
            Move::new(Square::C2, Square::C1, MoveFlag::QueenPromotion),
            Move::new(Square::C2, Square::C1, MoveFlag::RookPromotion),
            Move::new(Square::C2, Square::C1, MoveFlag::BishopPromotion),
            Move::new(Square::C2, Square::C1, MoveFlag::KnightPromotion),
            Move::new(Square::C2, Square::B1, MoveFlag::QueenPromotionCapture),
            Move::new(Square::C2, Square::B1, MoveFlag::RookPromotionCapture),
            Move::new(Square::C2, Square::B1, MoveFlag::BishopPromotionCapture),
            Move::new(Square::C2, Square::B1, MoveFlag::KnightPromotionCapture),
            Move::new(Square::B5, Square::A4, MoveFlag::Capture),
            Move::new(Square::A5, Square::B4, MoveFlag::Capture),
        ];

        let mut moves = Vec::new();
        generate_pawn_moves(MoveStage::All, &position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn castle_free_white() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let expected_moves = &[
            Move::new(Square::E1, Square::G1, MoveFlag::KingsideCastle),
            Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle),
        ];

        let mut moves = Vec::new();
        generate_king_castles(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn castle_free_black() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();

        let expected_moves = &[
            Move::new(Square::E8, Square::G8, MoveFlag::KingsideCastle),
            Move::new(Square::E8, Square::C8, MoveFlag::QueensideCastle),
        ];

        let mut moves = Vec::new();
        generate_king_castles(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn castle_blocked_white() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/1R2K1NR w Kkq - 0 1").unwrap();

        let mut moves = Vec::new();
        generate_king_castles(&position, &mut moves);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn castle_blocked_black() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();

        let mut moves = Vec::new();
        generate_king_castles(&position, &mut moves);

        assert_eq!(moves.len(), 0);
    }
}
