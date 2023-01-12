use crate::position::{Color, Piece, Position, Square};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub is_capture: bool,
    pub is_enpassant: bool,
    pub is_castle: bool,
    pub promoted_to: Option<Piece>,
}

impl Move {
    fn new(from: Square, to: Square, is_capture: bool) -> Move {
        Move {
            from,
            to,
            is_capture,
            is_enpassant: false,
            is_castle: false,
            promoted_to: None,
        }
    }

    fn unchecked_directions(piece: &Piece) -> Vec<(i8, i8)> {
        match piece {
            Piece::Pawn => vec![],
            Piece::Rook => vec![(-1, 0), (1, 0), (0, -1), (0, 1)],
            Piece::Knight => {
                vec![
                    (-2, -1),
                    (-2, 1),
                    (-1, -2),
                    (-1, 2),
                    (1, -2),
                    (1, 2),
                    (2, -1),
                    (2, 1),
                ]
            }
            Piece::Bishop => vec![(-1, -1), (-1, 1), (1, -1), (1, 1)],
            Piece::Queen | Piece::King => {
                vec![
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                ]
            }
        }
    }

    fn pseudo_moves_per_square_regular(
        position: &Position,
        from: &Square,
        crawl: bool,
    ) -> Vec<Move> {
        let (piece, color) = position.at(from).unwrap();
        let mut moves = Vec::new();
        for (row, col) in Move::unchecked_directions(&piece) {
            let mut to = Square {
                row: (from.row as i8 + row) as u8,
                col: (from.col as i8 + col) as u8,
            };
            while to.row < 8 && to.col < 8 {
                if position.at(&to).is_none() {
                    moves.push(Move::new(from.to_owned(), to, false));
                } else {
                    let (_, to_color) = position.at(&to).unwrap();
                    if to_color != color {
                        moves.push(Move::new(from.to_owned(), to, true));
                    }
                    break;
                }
                if !crawl {
                    break;
                }
                to.row = (to.row as i8 + row) as u8;
                to.col = (to.col as i8 + col) as u8;
            }
        }
        moves
    }

    fn pseudo_moves_from_square(position: &Position, from: &Square) -> Vec<Move> {
        if position.at(from).is_none() {
            return vec![];
        }

        let (piece, color) = position.at(from).unwrap();
        if color != position.next_to_move {
            return vec![];
        }

        match piece {
            Piece::Bishop | Piece::Rook | Piece::Queen => {
                Move::pseudo_moves_per_square_regular(position, from, true)
            }
            Piece::Knight => Move::pseudo_moves_per_square_regular(position, from, false),
            Piece::King => {
                let mut moves = Move::pseudo_moves_per_square_regular(position, from, false);
                match color {
                    Color::White => {
                        if position.castling_rights.white_kingside
                            && position.board[0][5].is_none()
                            && position.board[0][6].is_none()
                        {
                            if let Some((Piece::Rook, Color::Black)) = position.board[0][7] {
                                moves.push(Move {
                                    from: from.to_owned(),
                                    to: Square { row: 0, col: 6 },
                                    is_capture: false,
                                    is_enpassant: false,
                                    is_castle: true,
                                    promoted_to: None,
                                });
                            }
                        }
                        if position.castling_rights.white_queenside
                            && position.board[0][3].is_none()
                            && position.board[0][2].is_none()
                            && position.board[0][1].is_none()
                        {
                            if let Some((Piece::Rook, Color::Black)) = position.board[0][0] {
                                moves.push(Move {
                                    from: from.to_owned(),
                                    to: Square { row: 0, col: 2 },
                                    is_capture: false,
                                    is_enpassant: false,
                                    is_castle: true,
                                    promoted_to: None,
                                });
                            }
                        }
                    }
                    Color::Black => {
                        if position.castling_rights.black_kingside
                            && position.board[7][5].is_none()
                            && position.board[7][6].is_none()
                        {
                            if let Some((Piece::Rook, Color::White)) = position.board[7][7] {
                                moves.push(Move {
                                    from: from.to_owned(),
                                    to: Square { row: 7, col: 6 },
                                    is_capture: false,
                                    is_enpassant: false,
                                    is_castle: true,
                                    promoted_to: None,
                                });
                            }
                        }
                        if position.castling_rights.black_queenside
                            && position.board[7][3].is_none()
                            && position.board[7][2].is_none()
                            && position.board[7][1].is_none()
                        {
                            if let Some((Piece::Rook, Color::White)) = position.board[7][0] {
                                moves.push(Move {
                                    from: from.to_owned(),
                                    to: Square { row: 7, col: 2 },
                                    is_capture: false,
                                    is_enpassant: false,
                                    is_castle: true,
                                    promoted_to: None,
                                });
                            }
                        }
                    }
                }
                moves
            }
            Piece::Pawn => {
                let front_direction = match color {
                    Color::White => (1, 0),
                    Color::Black => (-1, 0),
                };
                let mut moves = vec![Move::new(
                    from.to_owned(),
                    Square {
                        row: (from.row as i8 + front_direction.0) as u8,
                        col: from.col,
                    },
                    false,
                )];

                if from.row == 1 && color == Color::White {
                    if position.board[3][from.col as usize].is_none() {
                        moves.push(Move::new(
                            from.to_owned(),
                            Square {
                                row: 3,
                                col: moves[0].to.col,
                            },
                            false,
                        ));
                    }
                } else if from.row == 6 && color == Color::Black {
                    if position.board[4][from.col as usize].is_none() {
                        moves.push(Move::new(
                            from.to_owned(),
                            Square {
                                row: 4,
                                col: moves[0].to.col,
                            },
                            false,
                        ));
                    }
                }

                let capture_squares = vec![
                    (moves[0].from.row, moves[0].from.col as i8 - 1),
                    (moves[0].from.row, moves[0].from.col as i8 + 1),
                ];
                for (row, col) in capture_squares {
                    let to = Square {
                        row: (from.row as i8 + row as i8) as u8,
                        col: (from.col as i8 + col as i8) as u8,
                    };
                    if to.row < 8 && to.col < 8 {
                        if position.at(&to).is_some() {
                            let (_, to_color) = position.at(&to).unwrap();
                            if to_color != color {
                                moves.push(Move::new(from.to_owned(), to, false));
                            }
                        } else if position.en_passant_square.is_some()
                            && to == position.en_passant_square.unwrap()
                        {
                            moves.push(Move::new(from.to_owned(), to, false));
                        }
                    }
                }

                moves
            }
        }
    }

    pub fn possible_moves(position: &Position) -> Vec<Move> {
        let mut moves = Vec::new();
        for row in 0..8 {
            for col in 0..8 {
                let from = Square { row, col };
                let moves_from_square = Move::pseudo_moves_from_square(position, &from);
                moves.extend(moves_from_square);
            }
        }
        moves
    }

    pub fn make(&self, position: &Position) -> Position {
        let (piece, color) = position.at(&self.from).unwrap();
        let mut new_board = position.board.clone();

        new_board[self.to.row as usize][self.to.col as usize] = Some((piece, color));
        new_board[self.from.row as usize][self.from.col as usize] = None;

        let mut new_en_passant_square = None;
        match piece {
            Piece::Pawn => {
                if self.from.row == 1 && self.to.row == 3 {
                    new_en_passant_square = Some(Square {
                        row: 2,
                        col: self.to.col,
                    });
                } else if self.from.row == 6 && self.to.row == 4 {
                    new_en_passant_square = Some(Square {
                        row: 5,
                        col: self.to.col,
                    });
                }
            }
            _ => {}
        }

        let mut new_castling_rights = position.castling_rights.clone();
        match piece {
            Piece::King => {
                if color == Color::White {
                    new_castling_rights.white_kingside = false;
                    new_castling_rights.white_queenside = false;
                } else {
                    new_castling_rights.black_kingside = false;
                    new_castling_rights.black_queenside = false;
                }
            }
            Piece::Rook => {
                if color == Color::White {
                    if self.from == (Square { row: 0, col: 0 }) {
                        new_castling_rights.white_queenside = false;
                    } else if self.from == (Square { row: 0, col: 7 }) {
                        new_castling_rights.white_kingside = false;
                    }
                } else {
                    if self.from == (Square { row: 7, col: 0 }) {
                        new_castling_rights.black_queenside = false;
                    } else if self.from == (Square { row: 7, col: 7 }) {
                        new_castling_rights.black_kingside = false;
                    }
                }
            }
            _ => {}
        }

        let is_capture = position.at(&self.to).is_some();
        let is_pawn_move = match piece {
            Piece::Pawn => true,
            _ => false,
        };
        let new_position = Position {
            board: new_board,
            en_passant_square: new_en_passant_square,
            castling_rights: new_castling_rights,
            half_move_number: if is_capture || is_pawn_move {
                0
            } else {
                position.half_move_number + 1
            },
            full_move_number: if color == Color::Black {
                position.full_move_number + 1
            } else {
                position.full_move_number
            },
            next_to_move: color.opposing(),
        };
        new_position
    }

    pub fn to_algebraic(&self) -> String {
        let mut notation = String::new();
        notation.push_str(&self.from.to_algebraic());
        notation.push_str(&self.to.to_algebraic());
        notation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_pawn_move() {
        let position = Position::new();
        let move_ = Move::new(Square { row: 1, col: 0 }, Square { row: 3, col: 0 }, false);
        let new_position = move_.make(&position);
        assert_eq!(new_position.board[3][0], Some((Piece::Pawn, Color::White)));
        assert_eq!(new_position.board[1][0], None);
    }

    #[test]
    fn start_knight_move() {
        let position = Position::new();
        let move_ = Move::new(Square { row: 0, col: 6 }, Square { row: 2, col: 5 }, false);
        let new_position = move_.make(&position);
        assert_eq!(
            new_position.board[2][5],
            Some((Piece::Knight, Color::White))
        );
        assert_eq!(new_position.board[0][6], None);
    }

    #[test]
    fn possible_moves_from_start() {
        let position = Position::new();
        let moves = Move::possible_moves(&position);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn possible_positions_halfmove2() {}
}
