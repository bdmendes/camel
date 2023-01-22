use std::fmt;

use super::{
    piece::{Color, Piece, DOWN, LEFT, RIGHT, UP},
    CastlingRights, Position, Square, BOARD_SIZE, ROW_SIZE,
};

const CASTLE_SQUARES: [[u8; 5]; 4] = [
    [4, 5, 6, 7, BOARD_SIZE],     // White kingside
    [4, 3, 2, 1, 0],              // White queenside
    [60, 61, 62, 63, BOARD_SIZE], // Black kingside
    [60, 59, 58, 57, 56],         // Black queenside
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub enpassant: bool,
    pub castle: bool,
    pub promotion: Option<Piece<Color>>,
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

impl Move {
    pub fn new(from: Square, to: Square, capture: bool) -> Move {
        Move {
            from,
            to,
            capture,
            enpassant: false,
            castle: false,
            promotion: None,
        }
    }
}

fn generate_regular_moves_from_square(
    position: &Position,
    square: &Square,
    directions: Vec<i8>,
) -> Vec<Move> {
    let piece = position.at(square).unwrap();
    let color = piece.color();
    let crawls = piece.is_crawling();
    let mut moves = Vec::new();

    for offset in directions {
        let mut current_offset = offset;
        let mut last_column = square.col();

        loop {
            let to_index = (square.index as i8 + current_offset) as u8;
            let current_col = to_index % ROW_SIZE;
            let out_of_bounds =
                to_index >= BOARD_SIZE || (current_col as i8 - last_column as i8).abs() > 2;
            if out_of_bounds {
                break;
            }

            let to_piece = position.board[to_index as usize];
            if to_piece.is_none() {
                moves.push(Move::new(*square, Square { index: to_index }, false));
                if crawls {
                    last_column = current_col;
                    current_offset += offset;
                    continue;
                }
            } else if to_piece.unwrap().color() != color {
                moves.push(Move::new(*square, Square { index: to_index }, true));
            }
            break;
        }
    }
    moves
}

fn pseudo_legal_moves_from_square(position: &Position, square: &Square) -> Vec<Move> {
    let piece = position.at(square).unwrap();
    match piece {
        Piece::Bishop(_) | Piece::Knight(_) | Piece::Rook(_) | Piece::Queen(_) => {
            generate_regular_moves_from_square(position, square, piece.unchecked_directions())
        }
        Piece::Pawn(color) => {
            let mut directions = piece.unchecked_directions();
            let front_direction = directions[0];
            let row = square.row();

            // Do not advance if there is a piece in front
            if (position.board[(square.index as i8 + front_direction) as usize]).is_some() {
                directions.pop();
            }

            // Move two squares on first pawn move
            if (row == 1 && color == Color::White) || (row == 6 && color == Color::Black) {
                if (position.board[(square.index as i8 + front_direction * 2) as usize]).is_none()
                    && (position.board[(square.index as i8 + front_direction) as usize]).is_none()
                {
                    directions.push(front_direction * 2);
                }
            }

            // Capture squares, if they are occupied by an opponent piece
            for capture_direction in [
                (front_direction + LEFT) as i8,
                (front_direction + RIGHT) as i8,
            ] {
                let capture_square = (square.index as i8 + capture_direction) as u8;
                if capture_square > BOARD_SIZE {
                    break;
                }
                let to_piece = position.board[capture_square as usize];
                let is_opponent_piece = to_piece.is_some() && to_piece.unwrap().color() != color;
                let is_en_passant = position.en_passant_square.is_some()
                    && position.en_passant_square.unwrap().index == capture_square;
                if is_opponent_piece || is_en_passant {
                    directions.push(capture_direction);
                }
            }

            // Handle promotion and en passant
            let mut moves = generate_regular_moves_from_square(position, square, directions);
            let mut under_promotion_moves = Vec::<Move>::new();
            for move_ in &mut moves {
                let row = move_.to.row();
                if row == 0 || row == 7 {
                    move_.promotion = Some(Piece::Queen(color));
                    for promotion_piece in [
                        Piece::Knight(color),
                        Piece::Bishop(color),
                        Piece::Rook(color),
                    ] {
                        let mut promotion_move = move_.clone();
                        promotion_move.promotion = Some(promotion_piece);
                        under_promotion_moves.push(promotion_move);
                    }
                } else if position.en_passant_square.is_some()
                    && move_.to == position.en_passant_square.unwrap()
                {
                    move_.enpassant = true;
                }
            }
            moves.extend(under_promotion_moves);

            moves
        }
        Piece::King(color) => {
            let mut moves =
                generate_regular_moves_from_square(position, square, piece.unchecked_directions());

            // Handle castling
            for squares in CASTLE_SQUARES {
                if squares[0] != square.index {
                    continue;
                }

                if position.board[squares[1] as usize].is_some()
                    || position.board[squares[2] as usize].is_some()
                {
                    continue;
                }

                if squares[4] >= BOARD_SIZE {
                    let can_castle_kingside = position.castling_rights
                        & (CastlingRights::WHITE_KINGSIDE | CastlingRights::BLACK_KINGSIDE);
                    if can_castle_kingside.is_empty() {
                        continue;
                    }

                    if position.board[squares[3] as usize].is_none()
                        || position.board[squares[3] as usize].unwrap() != Piece::Rook(color)
                    {
                        continue;
                    }
                } else {
                    let can_castle_queenside = position.castling_rights
                        & (CastlingRights::WHITE_QUEENSIDE | CastlingRights::BLACK_QUEENSIDE);
                    if can_castle_queenside.is_empty() {
                        continue;
                    }

                    if position.board[squares[3] as usize].is_some()
                        || position.board[squares[4] as usize].is_none()
                        || position.board[squares[4] as usize].unwrap() != Piece::Rook(color)
                    {
                        continue;
                    }
                }

                moves.push(Move {
                    from: *square,
                    to: Square { index: squares[2] },
                    capture: false,
                    promotion: None,
                    enpassant: false,
                    castle: true,
                });
            }

            moves
        }
    }
}

pub fn pseudo_legal_moves(position: &Position, to_move: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    for index in 0..BOARD_SIZE {
        let piece = position.board[index as usize];
        if piece.is_none() || piece.unwrap().color() != to_move {
            continue;
        }
        moves.extend(pseudo_legal_moves_from_square(position, &Square { index }));
    }
    moves
}

pub fn make_move(position: &Position, move_: &Move) -> Position {
    let mut new_board = position.board.clone();
    new_board[move_.to.index as usize] = new_board[move_.from.index as usize];
    new_board[move_.from.index as usize] = None;

    // En passant
    if move_.enpassant {
        let capture_square = match position.to_move {
            Color::White => move_.to.index as i8 + DOWN,
            Color::Black => move_.to.index as i8 + UP,
        };
        new_board[capture_square as usize] = None;
    }

    // Promotion
    if let Some(promotion_piece) = move_.promotion {
        new_board[move_.to.index as usize] = Some(promotion_piece);
    }

    // Castling
    let mut new_castling_rights = position.castling_rights.clone();
    if move_.castle {
        if let Some(piece) = new_board[move_.to.index as usize] {
            if piece == Piece::King(position.to_move) {
                match position.to_move {
                    Color::White => {
                        new_castling_rights &=
                            !(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
                        if move_.to.index == 6 {
                            new_board[5] = new_board[7];
                            new_board[7] = None;
                        } else if move_.to.index == 2 {
                            new_board[3] = new_board[0];
                            new_board[0] = None;
                        }
                    }
                    Color::Black => {
                        new_castling_rights &=
                            !(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
                        if move_.to.index == 62 {
                            new_board[61] = new_board[63];
                            new_board[63] = None;
                        } else if move_.to.index == 58 {
                            new_board[59] = new_board[56];
                            new_board[56] = None;
                        }
                    }
                }
            }
        }
    } else if let Some(piece) = position.board[move_.from.index as usize] {
        match piece {
            Piece::King(_) => match position.to_move {
                Color::White => {
                    new_castling_rights &=
                        !(CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE);
                }
                Color::Black => {
                    new_castling_rights &=
                        !(CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE);
                }
            },
            Piece::Rook(Color::White) => {
                if move_.from.index == 0 {
                    new_castling_rights &= !CastlingRights::WHITE_QUEENSIDE;
                } else if move_.from.index == 7 {
                    new_castling_rights &= !CastlingRights::WHITE_KINGSIDE;
                }
            }
            Piece::Rook(Color::Black) => {
                if move_.from.index == 56 {
                    new_castling_rights &= !CastlingRights::BLACK_QUEENSIDE;
                } else if move_.from.index == 63 {
                    new_castling_rights &= !CastlingRights::BLACK_KINGSIDE;
                }
            }
            _ => {}
        }
    }

    Position {
        board: new_board,
        to_move: position.to_move.opposing(),
        castling_rights: new_castling_rights,
        en_passant_square: match position.at(&move_.from).unwrap() {
            Piece::Pawn(_) => {
                if (move_.to.index as i8 - move_.from.index as i8).abs() == 2 * UP {
                    Some(Square {
                        index: (move_.to.index + move_.from.index) / 2,
                    })
                } else {
                    None
                }
            }
            _ => None,
        },
        half_move_number: if move_.capture
            || match position.at(&move_.from).unwrap() {
                Piece::Pawn(_) => true,
                _ => false,
            } {
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

pub fn legal_moves(position: &Position) -> Vec<Move> {
    let mut moves = pseudo_legal_moves(position, position.to_move);
    moves.retain(|move_| {
        if let Some(piece) = position.at(&move_.from) {
            if piece == Piece::King(position.to_move) && move_.castle {
                if position.is_check(
                    position.to_move,
                    Some(Square {
                        index: (move_.to.index + move_.from.index) / 2,
                    }),
                ) {
                    return false;
                }
            }
        }

        let new_position = make_move(position, move_);
        !new_position.is_check(position.to_move, None)
    });
    moves
}

#[cfg(test)]
mod tests {
    #[test]
    fn legal_moves_from_start_position() {
        let position = super::Position::new();
        let moves = super::legal_moves(&position);
        assert_eq!(moves.len(), 20);
    }
}
