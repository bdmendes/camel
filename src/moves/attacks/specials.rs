use crate::{
    moves::{gen::MoveDirection, Move, MoveFlag},
    position::{bitboard::Bitboard, board::Piece, square::Square, CastlingRights, Color, Position},
};

const PAWN_WEST_EDGE_FILE: Bitboard = Bitboard::new(0x01_01_01_01_01_01_01_01);
const PAWN_EAST_EDGE_FILE: Bitboard = Bitboard::new(0x80_80_80_80_80_80_80_80);

pub fn generate_pawn_moves<const QUIESCE: bool>(position: &Position, mut moves: &mut Vec<Move>) {
    let occupancy = position.board.occupancy_bb_all();
    let occupancy_them = position.board.occupancy_bb(position.side_to_move.opposite())
        | position.en_passant_square.map_or(Bitboard::new(0), |sq| Bitboard::new(1 << sq as u8));
    let our_pawns =
        position.board.pieces_bb(Piece::Pawn) & position.board.occupancy_bb(position.side_to_move);

    let direction = if position.side_to_move == Color::White {
        MoveDirection::NORTH
    } else {
        MoveDirection::SOUTH
    };

    // Single push
    let mut single_push_pawns = our_pawns.shift(direction) & !occupancy;
    let single_push_pawns_cpy = single_push_pawns.clone();
    while let Some(to_square) = single_push_pawns.pop_lsb() {
        let from_square = Square::try_from((to_square as i8 - direction) as u8).unwrap();
        push_pawn_move::<QUIESCE>(occupancy, None, &mut moves, from_square, to_square);
    }

    if !QUIESCE {
        // Double push
        let third_row_bb = Bitboard::new(if position.side_to_move == Color::White {
            0x00_00_00_00_00_FF_00_00
        } else {
            0x00_00_FF_00_00_00_00_00
        });
        let mut double_push_pawns =
            (single_push_pawns_cpy & third_row_bb).shift(direction) & !occupancy;

        while let Some(to_square) = double_push_pawns.pop_lsb() {
            let from_square =
                Square::try_from((to_square as i8 - direction - direction) as u8).unwrap();
            moves.push(Move::new(from_square, to_square, MoveFlag::DoublePawnPush));
        }
    }

    // West capture
    let mut west_pawns =
        (our_pawns & !PAWN_WEST_EDGE_FILE).shift(direction + MoveDirection::WEST) & occupancy_them;
    while let Some(to_square) = west_pawns.pop_lsb() {
        let from_square =
            Square::try_from((to_square as i8 - direction - MoveDirection::WEST) as u8).unwrap();
        push_pawn_move::<QUIESCE>(
            occupancy,
            position.en_passant_square,
            &mut moves,
            from_square,
            to_square,
        );
    }

    // East capture
    let mut east_pawns =
        (our_pawns & !PAWN_EAST_EDGE_FILE).shift(direction + MoveDirection::EAST) & occupancy_them;
    while let Some(to_square) = east_pawns.pop_lsb() {
        let from_square =
            Square::try_from((to_square as i8 - direction - MoveDirection::EAST) as u8).unwrap();
        push_pawn_move::<QUIESCE>(
            occupancy,
            position.en_passant_square,
            &mut moves,
            from_square,
            to_square,
        );
    }
}

fn push_pawn_move<const QUIESCE: bool>(
    occupancy: Bitboard,
    en_passant_square: Option<Square>,
    moves: &mut Vec<Move>,
    from_square: Square,
    to_square: Square,
) {
    let is_promotion = to_square.rank() == 0 || to_square.rank() == 7;

    if is_promotion {
        push_pawn_promotion(occupancy, moves, from_square, to_square);
    } else {
        let is_capture = occupancy.is_set(to_square) || en_passant_square == Some(to_square);
        if QUIESCE && !is_capture {
            return;
        }
        moves.push(Move::new(
            from_square,
            to_square,
            if en_passant_square == Some(to_square) {
                MoveFlag::EnPassantCapture
            } else if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            },
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
        Color::White => generate_white_king_castles(position, moves.as_mut()),
        Color::Black => generate_black_king_castles(position, moves.as_mut()),
    }
}

fn generate_white_king_castles(position: &Position, moves: &mut Vec<Move>) {
    if position.castling_rights.contains(CastlingRights::WHITE_KINGSIDE)
        && position.board.piece_at(Square::E1) == Some((Piece::King, Color::White))
        && position.board.piece_at(Square::H1) == Some((Piece::Rook, Color::White))
        && position.board.piece_at(Square::F1) == None
        && position.board.piece_at(Square::G1) == None
    {
        moves.push(Move::new(Square::E1, Square::G1, MoveFlag::KingsideCastle));
    }

    if position.castling_rights.contains(CastlingRights::WHITE_QUEENSIDE)
        && position.board.piece_at(Square::E1) == Some((Piece::King, Color::White))
        && position.board.piece_at(Square::A1) == Some((Piece::Rook, Color::White))
        && position.board.piece_at(Square::B1) == None
        && position.board.piece_at(Square::C1) == None
        && position.board.piece_at(Square::D1) == None
    {
        moves.push(Move::new(Square::E1, Square::C1, MoveFlag::QueensideCastle));
    }
}

fn generate_black_king_castles(position: &Position, moves: &mut Vec<Move>) {
    if position.castling_rights.contains(CastlingRights::BLACK_KINGSIDE)
        && position.board.piece_at(Square::E8) == Some((Piece::King, Color::Black))
        && position.board.piece_at(Square::H8) == Some((Piece::Rook, Color::Black))
        && position.board.piece_at(Square::F8) == None
        && position.board.piece_at(Square::G8) == None
    {
        moves.push(Move::new(Square::E8, Square::G8, MoveFlag::KingsideCastle));
    }

    if position.castling_rights.contains(CastlingRights::BLACK_QUEENSIDE)
        && position.board.piece_at(Square::E8) == Some((Piece::King, Color::Black))
        && position.board.piece_at(Square::A8) == Some((Piece::Rook, Color::Black))
        && position.board.piece_at(Square::B8) == None
        && position.board.piece_at(Square::C8) == None
        && position.board.piece_at(Square::D8) == None
    {
        moves.push(Move::new(Square::E8, Square::C8, MoveFlag::QueensideCastle));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{attacks::specials::generate_pawn_moves, Move, MoveFlag},
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

        let mut moves = Vec::new();
        generate_pawn_moves::<false>(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(&mov));
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

        let mut moves = Vec::new();
        generate_pawn_moves::<false>(&position, &mut moves);

        for mov in &moves {
            assert!(expected_moves.contains(&mov));
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

        let mut moves = Vec::new();
        generate_king_castles(&position, moves.as_mut());

        for mov in &moves {
            assert!(expected_moves.contains(&mov));
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

        let mut moves = Vec::new();
        generate_king_castles(&position, moves.as_mut());

        for mov in &moves {
            assert!(expected_moves.contains(&mov));
        }

        assert_eq!(moves.len(), expected_moves.len());
    }

    #[test]
    fn castle_blocked_white() {
        let position =
            Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/1R2K1NR w Kkq - 0 1").unwrap();

        let mut moves = Vec::new();
        generate_king_castles(&position, moves.as_mut());

        assert_eq!(moves.len(), 0);
    }
}
