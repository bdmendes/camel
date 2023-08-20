use crate::{
    moves::{
        gen::{square_attacked_by, MoveDirection},
        Move, MoveFlag, MoveVec,
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

pub fn pawn_attacks(board: &Board, color: Color) -> Bitboard {
    let our_pawns = board.pieces_bb(Piece::Pawn) & board.occupancy_bb(color);
    let direction = MoveDirection::pawn_direction(color);

    (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST)
        | (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST)
}

pub fn generate_pawn_moves<const QUIESCE: bool>(position: &Position, moves: &mut MoveVec) {
    let occupancy = position.board.occupancy_bb_all();
    let occupancy_them = position.board.occupancy_bb(position.side_to_move.opposite());
    let our_pawns =
        position.board.pieces_bb(Piece::Pawn) & position.board.occupancy_bb(position.side_to_move);

    let direction = MoveDirection::pawn_direction(position.side_to_move);

    // Single push
    let single_push_pawns = our_pawns.shift(direction) & !occupancy;
    for to_square in single_push_pawns {
        let from_square = Square::from((to_square as i8 - direction) as u8).unwrap();
        push_pawn_move::<QUIESCE>(occupancy, moves, from_square, to_square);
    }

    if !QUIESCE {
        // Double push
        let third_row_bb =
            Bitboard::rank_mask(if position.side_to_move == Color::White { 2 } else { 5 });
        let double_push_pawns = (single_push_pawns & third_row_bb).shift(direction) & !occupancy;

        for to_square in double_push_pawns {
            let from_square =
                Square::from((to_square as i8 - direction - direction) as u8).unwrap();
            moves.push(Move::new(from_square, to_square, MoveFlag::DoublePawnPush));
        }
    }

    // West capture
    let west_pawns =
        (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST) & occupancy_them;
    for to_square in west_pawns {
        let from_square =
            Square::from((to_square as i8 - direction - MoveDirection::WEST) as u8).unwrap();
        push_pawn_move::<QUIESCE>(occupancy_them, moves, from_square, to_square);
    }

    // East capture
    let east_pawns =
        (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST) & occupancy_them;
    for to_square in east_pawns {
        let from_square =
            Square::from((to_square as i8 - direction - MoveDirection::EAST) as u8).unwrap();
        push_pawn_move::<QUIESCE>(occupancy_them, moves, from_square, to_square);
    }

    // En passant
    if let Some(en_passant_square) = position.en_passant_square {
        let ep_bb = Bitboard::new(1 << en_passant_square as u8);

        let west_pawns =
            (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST) & ep_bb;
        for to_square in west_pawns {
            let from_square =
                Square::from((to_square as i8 - direction - MoveDirection::WEST) as u8).unwrap();
            moves.push(Move::new(from_square, to_square, MoveFlag::EnPassantCapture));
        }

        let east_pawns =
            (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST) & ep_bb;
        for to_square in east_pawns {
            let from_square =
                Square::from((to_square as i8 - direction - MoveDirection::EAST) as u8).unwrap();
            moves.push(Move::new(from_square, to_square, MoveFlag::EnPassantCapture));
        }
    }
}

fn push_pawn_move<const QUIESCE: bool>(
    occupancy_them: Bitboard,
    moves: &mut MoveVec,
    from_square: Square,
    to_square: Square,
) {
    let is_promotion = to_square.rank() == 0 || to_square.rank() == 7;

    if is_promotion {
        push_pawn_promotion::<QUIESCE>(occupancy_them, moves, from_square, to_square);
    } else {
        let is_capture = occupancy_them.is_set(to_square);
        if QUIESCE && !is_capture {
            return;
        }
        moves.push(Move::new(
            from_square,
            to_square,
            if is_capture { MoveFlag::Capture } else { MoveFlag::Quiet },
        ));
    }
}

fn push_pawn_promotion<const QUIESCE: bool>(
    occupancy: Bitboard,
    moves: &mut MoveVec,
    from_square: Square,
    to_square: Square,
) {
    let is_capture = occupancy.is_set(to_square);

    moves.push(Move::new(
        from_square,
        to_square,
        if is_capture { MoveFlag::QueenPromotionCapture } else { MoveFlag::QueenPromotion },
    ));

    if QUIESCE {
        return;
    }

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

pub fn generate_king_castles(position: &Position, moves: &mut MoveVec) {
    match position.side_to_move {
        Color::White => generate_white_king_castles(position, moves),
        Color::Black => generate_black_king_castles(position, moves),
    }
}

fn generate_white_king_castles(position: &Position, moves: &mut MoveVec) {
    if position.castling_rights.contains(CastlingRights::WHITE_KINGSIDE)
        && position.board.piece_color_at(Square::E1) == Some((Piece::King, Color::White))
        && position.board.piece_color_at(Square::H1) == Some((Piece::Rook, Color::White))
        && position.board.color_at(Square::F1).is_none()
        && position.board.color_at(Square::G1).is_none()
        && !square_attacked_by(&position.board, Square::E1, Color::Black)
        && !square_attacked_by(&position.board, Square::F1, Color::Black)
        && !square_attacked_by(&position.board, Square::G1, Color::Black)
    {
        moves.push(Move::new(Square::E1, Square::G1, MoveFlag::KingsideCastle));
    }

    if position.castling_rights.contains(CastlingRights::WHITE_QUEENSIDE)
        && position.board.piece_color_at(Square::E1) == Some((Piece::King, Color::White))
        && position.board.piece_color_at(Square::A1) == Some((Piece::Rook, Color::White))
        && position.board.color_at(Square::B1).is_none()
        && position.board.color_at(Square::C1).is_none()
        && position.board.color_at(Square::D1).is_none()
        && !square_attacked_by(&position.board, Square::E1, Color::Black)
        && !square_attacked_by(&position.board, Square::D1, Color::Black)
        && !square_attacked_by(&position.board, Square::C1, Color::Black)
    {
        moves.push(Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle));
    }
}

fn generate_black_king_castles(position: &Position, moves: &mut MoveVec) {
    if position.castling_rights.contains(CastlingRights::BLACK_KINGSIDE)
        && position.board.piece_color_at(Square::E8) == Some((Piece::King, Color::Black))
        && position.board.piece_color_at(Square::H8) == Some((Piece::Rook, Color::Black))
        && position.board.color_at(Square::F8).is_none()
        && position.board.color_at(Square::G8).is_none()
        && !square_attacked_by(&position.board, Square::E8, Color::White)
        && !square_attacked_by(&position.board, Square::F8, Color::White)
        && !square_attacked_by(&position.board, Square::G8, Color::White)
    {
        moves.push(Move::new(Square::E8, Square::G8, MoveFlag::KingsideCastle));
    }

    if position.castling_rights.contains(CastlingRights::BLACK_QUEENSIDE)
        && position.board.piece_color_at(Square::E8) == Some((Piece::King, Color::Black))
        && position.board.piece_color_at(Square::A8) == Some((Piece::Rook, Color::Black))
        && position.board.color_at(Square::B8).is_none()
        && position.board.color_at(Square::C8).is_none()
        && position.board.color_at(Square::D8).is_none()
        && !square_attacked_by(&position.board, Square::E8, Color::White)
        && !square_attacked_by(&position.board, Square::D8, Color::White)
        && !square_attacked_by(&position.board, Square::C8, Color::White)
    {
        moves.push(Move::new(Square::E8, Square::C8, MoveFlag::QueensideCastle));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{attacks::specials::generate_pawn_moves, Move, MoveFlag, MoveVec},
        position::{square::Square, Position},
    };

    use super::generate_king_castles;

    #[test]
    fn pawn_moves_white() {
        let position =
            Position::from_fen("4r3/3P3p/8/pp3pp1/PP1pP1PP/8/2p2P2/1B6 w - - 0 1").unwrap();

        let expected_moves = vec![
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

        let mut moves = MoveVec::new();
        generate_pawn_moves::<false>(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn pawn_moves_black() {
        let position =
            Position::from_fen("4r3/3P3p/8/pp3pp1/PP1pP1PP/8/2p2P2/1B6 b - e3 0 1").unwrap();

        let expected_moves = vec![
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

        let mut moves = MoveVec::new();
        generate_pawn_moves::<false>(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn castle_free_white() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();

        let expected_moves = vec![
            Move::new(Square::E1, Square::G1, MoveFlag::KingsideCastle),
            Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle),
        ];

        let mut moves = MoveVec::new();
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

        let expected_moves = vec![
            Move::new(Square::E8, Square::G8, MoveFlag::KingsideCastle),
            Move::new(Square::E8, Square::C8, MoveFlag::QueensideCastle),
        ];

        let mut moves = MoveVec::new();
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

        let mut moves = MoveVec::new();
        generate_king_castles(&position, &mut moves);

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn castle_blocked_black() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();

        let mut moves = MoveVec::new();
        generate_king_castles(&position, &mut moves);

        assert_eq!(moves.len(), 0);
    }
}
