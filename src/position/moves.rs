use super::{
    piece::{Color, Piece, DOWN, UP},
    square::ROW_SIZE,
    CastlingRights, Position, Square, BOARD_SIZE,
};
use bitflags::bitflags;
use smallvec::SmallVec;
use std::fmt;

bitflags! {
    pub struct MoveFlags: u8 {
        const CAPTURE = 0b001;
        const ENPASSANT = 0b010;
        const CASTLE = 0b100;
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from.to_algebraic(),
            self.to.to_algebraic(),
            match self.promotion {
                Some(promotion) => promotion.to_char().to_string().to_lowercase(),
                None => "".to_owned(),
            }
        )
    }
}

type SmallMoveVec = SmallVec<[Move; 16]>;

impl Move {
    pub fn new(from: Square, to: Square, flags: MoveFlags) -> Move {
        Move { from, to, flags, promotion: None }
    }

    pub fn is_tactical(&self) -> bool {
        self.flags.contains(MoveFlags::CAPTURE) || self.promotion.is_some()
    }
}

fn generate_regular_moves_from_square(
    position: &Position,
    square: Square,
    directions: &[isize],
    only_captures: bool,
    faker_piece: Option<Piece>,
) -> SmallMoveVec {
    let piece = faker_piece.unwrap_or(position.board[square].unwrap());
    let color = piece.color();
    let slides = piece.is_sliding();
    let mut moves = SmallMoveVec::new();

    for &offset in directions {
        let mut current_offset = offset;
        let mut last_column = square.col() as isize;

        loop {
            let to_index = (square.0 as isize + current_offset) as usize;
            let current_col = (to_index % ROW_SIZE) as isize;
            if to_index >= BOARD_SIZE || (current_col - last_column).abs() > 2 {
                break;
            }

            if let Some(to_piece) = position.board[to_index] {
                if to_piece.color() != color {
                    moves.push(Move::new(square, to_index.into(), MoveFlags::CAPTURE));
                }
                break;
            } else {
                if !only_captures {
                    moves.push(Move::new(square, to_index.into(), MoveFlags::empty()));
                }
                if slides {
                    last_column = current_col;
                    current_offset += offset;
                    continue;
                }
                break;
            }
        }
    }
    moves
}

pub fn pseudo_legal_moves_from_square(
    position: &Position,
    square: Square,
    only_tactical: bool,
    faker_piece: Option<Piece>,
) -> SmallMoveVec {
    let piece = faker_piece.unwrap_or(position.board[square].unwrap());
    let color = piece.color();
    let mut moves = generate_regular_moves_from_square(
        position,
        square,
        piece.unchecked_directions(),
        only_tactical,
        faker_piece,
    );

    match piece {
        Piece::WP | Piece::BP => {
            // Do not advance if there is a piece in front; do not capture if there is no piece
            moves.retain(|move_| {
                let index_diff = (move_.to.0 as isize - move_.from.0 as isize).abs();
                if index_diff == UP {
                    !move_.flags.contains(MoveFlags::CAPTURE)
                } else if index_diff == UP + UP {
                    let row = move_.from.row();
                    let can_advance_two = ((row == 1 && color == Color::White)
                        || (row == 6 && color == Color::Black))
                        && !move_.flags.contains(MoveFlags::CAPTURE);
                    let jumped_piece =
                        position.board[Square((move_.from.0 + move_.to.0) / 2)].is_some();
                    can_advance_two && !jumped_piece
                } else {
                    move_.flags.contains(MoveFlags::CAPTURE)
                        || matches!(position.en_passant_square,
                            Some(en_passant_square) if
                                move_.to == en_passant_square)
                }
            });

            // Add promotion and en passant
            let curr_moves_len = moves.len();
            for i in 0..curr_moves_len {
                let mut move_ = &mut moves[i];
                let row = move_.to.row();
                if row == 0 || row == 7 {
                    let mut under_promotion_moves = SmallVec::<[Move; 3]>::new();
                    let promotion_pieces = if color == Color::White {
                        [Piece::WQ, Piece::WR, Piece::WB, Piece::WN]
                    } else {
                        [Piece::BQ, Piece::BR, Piece::BB, Piece::BN]
                    };
                    move_.promotion = Some(promotion_pieces[0]);
                    for i in 1..=3 {
                        let promotion_move = Move {
                            from: move_.from,
                            to: move_.to,
                            flags: move_.flags,
                            promotion: Some(promotion_pieces[i]),
                        };
                        under_promotion_moves.push(promotion_move);
                    }
                    moves.extend(under_promotion_moves);
                } else if let Some(en_passant_square) = position.en_passant_square {
                    if move_.to == en_passant_square {
                        move_.flags |= MoveFlags::ENPASSANT;
                    }
                }
            }
        }
        Piece::WK | Piece::BK if !only_tactical => {
            // Add castle moves
            let castle_squares = match color {
                Color::White => [
                    [4, 7], // White kingside
                    [4, 0], // White queenside
                ],
                Color::Black => [
                    [60, 63], // Black kingside
                    [60, 56], // Black queenside
                ],
            };

            for i in 0..2 {
                // Check castle rights
                let is_kingside = i == 0;
                let castle_rights = match color {
                    Color::White => match is_kingside {
                        true => CastlingRights::WHITE_KINGSIDE,
                        false => CastlingRights::WHITE_QUEENSIDE,
                    },
                    Color::Black => match is_kingside {
                        true => CastlingRights::BLACK_KINGSIDE,
                        false => CastlingRights::BLACK_QUEENSIDE,
                    },
                };
                if !position.castling_rights.contains(castle_rights) {
                    continue;
                }

                // Check if king and rook are in place
                let squares = &castle_squares[i];
                if let Some(king_piece) = position.board[squares[0]] {
                    if king_piece != piece {
                        continue;
                    }
                }
                let same_color_rook = match color {
                    Color::White => Piece::WR,
                    Color::Black => Piece::BR,
                };
                if let Some(rook_piece) = position.board[squares[1]] {
                    if rook_piece != same_color_rook {
                        continue;
                    }
                }

                // Check if squares between king and rook are empty
                let mut inbetween_squares = match squares[1] > squares[0] {
                    true => (squares[0] + 1)..squares[1],
                    false => (squares[1] + 1)..squares[0],
                };
                if inbetween_squares.any(|i| position.board[i].is_some()) {
                    continue;
                }

                moves.push(Move {
                    from: square,
                    to: (if is_kingside { squares[0] + 2 } else { squares[0] - 2 }).into(),
                    flags: MoveFlags::CASTLE,
                    promotion: None,
                });
            }
        }
        _ => {}
    }

    moves
}

pub fn pseudo_legal_moves(position: &Position, to_move: Color, only_tactical: bool) -> Vec<Move> {
    let mut moves = Vec::with_capacity(BOARD_SIZE);
    for index in 0..BOARD_SIZE {
        if let Some(piece) = position.board[index] {
            if piece.color() != to_move {
                continue;
            }
            moves.extend(pseudo_legal_moves_from_square(
                position,
                index.into(),
                only_tactical,
                None,
            ));
        }
    }
    moves
}

pub fn legal_moves(position: &Position, only_non_quiet: bool) -> Vec<Move> {
    let mut moves = pseudo_legal_moves(position, position.to_move, only_non_quiet);
    let king_index = position.board.0.iter().position(|piece| {
        piece
            == &Some(match position.to_move {
                Color::White => Piece::WK,
                Color::Black => Piece::BK,
            })
    });
    let king_square = match king_index {
        Some(index) => Square(index as u8),
        None => return moves,
    };
    let is_check = position_is_check(position, position.to_move, None);

    moves.retain(|move_| {
        let castle_passent_squares = if move_.flags.contains(MoveFlags::CASTLE) {
            Some([move_.from, Square((move_.to.0 + move_.from.0) / 2)])
        } else {
            None
        };

        let new_position = make_move(position, move_);

        let is_king_move = matches!(position.board[move_.from], Some(Piece::WK) | Some(Piece::BK));
        let was_blocking_king = !is_king_move
            && (move_.from.row() == king_square.row()
                || move_.from.col() == king_square.col()
                || move_.from.same_diagonal(king_square));
        if is_king_move || was_blocking_king || is_check {
            return !position_is_check(&new_position, position.to_move, castle_passent_squares);
        }
        true
    });

    moves
}

pub fn position_is_check(
    position: &Position,
    checked_player: Color,
    castle_passent_squares: Option<[Square; 2]>,
) -> bool {
    let opposing_color = checked_player.opposite();
    let opponent_moves =
        pseudo_legal_moves(position, opposing_color, castle_passent_squares.is_none());

    opponent_moves.iter().any(|move_| {
        if let Some(piece) = position.board[move_.to] {
            if (piece == Piece::WK && checked_player == Color::White)
                || (piece == Piece::BK && checked_player == Color::Black)
            {
                return true;
            }
        }
        if let Some([square1, square2]) = castle_passent_squares {
            if move_.to == square1 || move_.to == square2 {
                return true;
            }
        }
        false
    })
}

pub fn make_move(position: &Position, move_: &Move) -> Position {
    let mut new_board = position.board;
    new_board[move_.to] = new_board[move_.from];
    new_board[move_.from] = None;

    // En passant
    if move_.flags.contains(MoveFlags::ENPASSANT) {
        let capture_square = match position.to_move {
            Color::White => move_.to.0 as isize + DOWN,
            Color::Black => move_.to.0 as isize + UP,
        };
        new_board[capture_square as usize] = None;
    }

    // Promotion
    if let Some(promotion_piece) = move_.promotion {
        new_board[move_.to] = Some(promotion_piece);
    }

    // Castling
    let mut new_castling_rights = position.castling_rights;
    let piece = position.board[move_.from].unwrap();
    if move_.flags.contains(MoveFlags::CASTLE) {
        if piece == Piece::WK {
            new_castling_rights &=
                !(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
            if move_.to.0 == 6 {
                new_board[5] = new_board[7];
                new_board[7] = None;
            } else if move_.to.0 == 2 {
                new_board[3] = new_board[0];
                new_board[0] = None;
            }
        } else if piece == Piece::BK {
            new_castling_rights &=
                !(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
            if move_.to.0 == 62 {
                new_board[61] = new_board[63];
                new_board[63] = None;
            } else if move_.to.0 == 58 {
                new_board[59] = new_board[56];
                new_board[56] = None;
            }
        }
    } else {
        match piece {
            Piece::WK => {
                new_castling_rights &=
                    !(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
            }
            Piece::BK => {
                new_castling_rights &=
                    !(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
            }
            Piece::WR => {
                if move_.from.0 == 0 {
                    new_castling_rights &= !CastlingRights::WHITE_QUEENSIDE;
                } else if move_.from.0 == 7 {
                    new_castling_rights &= !CastlingRights::WHITE_KINGSIDE;
                }
            }
            Piece::BR => {
                if move_.from.0 == 56 {
                    new_castling_rights &= !CastlingRights::BLACK_QUEENSIDE;
                } else if move_.from.0 == 63 {
                    new_castling_rights &= !CastlingRights::BLACK_KINGSIDE;
                }
            }
            _ => {}
        }
    }

    Position {
        board: new_board,
        to_move: position.to_move.opposite(),
        castling_rights: new_castling_rights,
        en_passant_square: match position.board[move_.from] {
            Some(Piece::WP) | Some(Piece::BP) => {
                if (move_.from.row() == 1 && move_.to.row() == 3)
                    || (move_.from.row() == 6 && move_.to.row() == 4)
                {
                    Some(Square((move_.to.0 + move_.from.0) / 2))
                } else {
                    None
                }
            }
            _ => None,
        },
        half_move_number: if move_.flags.contains(MoveFlags::CAPTURE)
            || matches!(position.board[move_.from].unwrap(), Piece::WP | Piece::BP)
        {
            0
        } else {
            position.half_move_number + 1
        },
        full_move_number: if position.to_move == Color::Black {
            position.full_move_number + 1
        } else {
            position.full_move_number
        },
    }
}

pub fn make_null_move(position: &Position) -> Position {
    Position {
        board: position.board,
        to_move: position.to_move.opposite(),
        castling_rights: position.castling_rights,
        en_passant_square: None,
        half_move_number: position.half_move_number,
        full_move_number: position.full_move_number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_moves() {
        let position = super::Position::new();
        let moves = legal_moves(&position, false);
        assert_eq!(moves.len(), 20);
    }
}
