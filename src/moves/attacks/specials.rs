use crate::{
    moves::{Move, MoveFlag},
    position::{board::Piece, square::Square, CastlingRights, Color, Position},
};

pub fn generate_pawn_moves<const QUIESCE: bool>(position: &Position, mut moves: &Vec<Move>) {}

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
