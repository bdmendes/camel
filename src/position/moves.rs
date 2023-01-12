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

    fn new_promotion(from: Square, to: Square, is_capture: bool, promoted_to: Piece) -> Move {
        Move {
            from,
            to,
            is_capture,
            is_enpassant: false,
            is_castle: false,
            promoted_to: Some(promoted_to),
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

    fn pseudo_moves_from_square(
        position: &Position,
        from: &Square,
        next_color: &Color,
    ) -> Vec<Move> {
        if position.at(from).is_none() {
            return vec![];
        }

        let (piece, color) = position.at(from).unwrap();
        if &color != next_color {
            return vec![];
        }

        match piece {
            Piece::Bishop | Piece::Rook | Piece::Queen => {
                Move::pseudo_moves_per_square_regular(position, from, true)
            }
            Piece::Knight => Move::pseudo_moves_per_square_regular(position, from, false),
            Piece::King => {
                let mut moves = Move::pseudo_moves_per_square_regular(position, from, false);

                //abort castle if king is in check
                if color == position.next_to_move {
                    let current_opponent_moves =
                        Self::pseudo_legal_moves(position, &color.opposing());
                    for move_ in current_opponent_moves {
                        if (move_.to.row, move_.to.col) == (from.row, from.col) {
                            return moves;
                        }
                    }
                }

                // castle
                match color {
                    Color::White => {
                        if position.castling_rights.white_kingside
                            && position.board[0][5].is_none()
                            && position.board[0][6].is_none()
                        {
                            if let Some((Piece::Rook, Color::White)) = position.board[0][7] {
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
                            if let Some((Piece::Rook, Color::White)) = position.board[0][0] {
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
                            if let Some((Piece::Rook, Color::Black)) = position.board[7][7] {
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
                            if let Some((Piece::Rook, Color::Black)) = position.board[7][0] {
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
                let to_front = Square {
                    row: (from.row as i8 + front_direction.0) as u8,
                    col: from.col,
                };

                // regular move
                let mut moves = if position.at(&to_front).is_none() {
                    if (color == Color::White && to_front.row == 7)
                        || (color == Color::Black && to_front.row == 0)
                    {
                        vec![
                            Move::new_promotion(from.to_owned(), to_front, false, Piece::Queen),
                            Move::new_promotion(from.to_owned(), to_front, false, Piece::Rook),
                            Move::new_promotion(from.to_owned(), to_front, false, Piece::Bishop),
                            Move::new_promotion(from.to_owned(), to_front, false, Piece::Knight),
                        ]
                    } else {
                        vec![Move::new(from.to_owned(), to_front, false)]
                    }
                } else {
                    vec![]
                };

                // first pawn move
                if from.row == 1 && color == Color::White {
                    if position.board[3][from.col as usize].is_none()
                        && position.board[2][from.col as usize].is_none()
                    {
                        moves.push(Move::new(
                            from.to_owned(),
                            Square {
                                row: 3,
                                col: to_front.col,
                            },
                            false,
                        ));
                    }
                } else if from.row == 6 && color == Color::Black {
                    if position.board[4][from.col as usize].is_none()
                        && position.board[5][from.col as usize].is_none()
                    {
                        moves.push(Move::new(
                            from.to_owned(),
                            Square {
                                row: 4,
                                col: to_front.col,
                            },
                            false,
                        ));
                    }
                }

                // capture
                let capture_squares = vec![
                    (to_front.row, (from.col as i8 - 1) as u8),
                    (to_front.row, (from.col as i8 + 1) as u8),
                ];
                for (row, col) in capture_squares {
                    if row < 8 && col < 8 {
                        let to = Square { row, col };
                        if position.at(&to).is_some() {
                            let (_, to_color) = position.at(&to).unwrap();
                            if to_color != color {
                                if (color == Color::White && to.row == 7)
                                    || (color == Color::Black && to.row == 0)
                                {
                                    moves.push(Move::new_promotion(
                                        from.to_owned(),
                                        to,
                                        true,
                                        Piece::Queen,
                                    ));
                                    moves.push(Move::new_promotion(
                                        from.to_owned(),
                                        to,
                                        true,
                                        Piece::Rook,
                                    ));
                                    moves.push(Move::new_promotion(
                                        from.to_owned(),
                                        to,
                                        true,
                                        Piece::Bishop,
                                    ));
                                    moves.push(Move::new_promotion(
                                        from.to_owned(),
                                        to,
                                        true,
                                        Piece::Knight,
                                    ));
                                } else {
                                    moves.push(Move::new(from.to_owned(), to, true));
                                }
                            }
                        } else if position.en_passant_square.is_some()
                            && to == position.en_passant_square.unwrap()
                        {
                            moves.push(Move::new(from.to_owned(), to, true));
                        }
                    }
                }

                moves
            }
        }
    }

    pub fn pseudo_legal_moves(position: &Position, color: &Color) -> Vec<Move> {
        let mut moves = Vec::new();
        for row in 0..8 {
            for col in 0..8 {
                let from = Square { row, col };
                let moves_from_square = Move::pseudo_moves_from_square(position, &from, color);
                moves.extend(moves_from_square);
            }
        }
        moves
    }

    pub fn legal_moves(position: &Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let pseudo_legal_moves = Self::pseudo_legal_moves(position, &position.next_to_move);

        for move_ in pseudo_legal_moves {
            let new_position = move_.make(position);
            let possible_moves =
                Self::pseudo_legal_moves(&new_position, &position.next_to_move.opposing());

            let mut is_legal = true;
            for possible_move in possible_moves {
                match new_position.at(&possible_move.to) {
                    Some((Piece::King, _)) => {
                        is_legal = false;
                        break;
                    }
                    _ => {}
                }
            }

            if is_legal {
                moves.push(move_);
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
                if self.is_enpassant {
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

                    if color == Color::White {
                        new_board[self.to.row as usize - 1][self.to.col as usize] = None;
                    } else {
                        new_board[self.to.row as usize + 1][self.to.col as usize] = None;
                    }
                }

                if (self.promoted_to.is_some() && self.to.row == 7)
                    || (self.promoted_to.is_some() && self.to.row == 0)
                {
                    new_board[self.to.row as usize][self.to.col as usize] =
                        Some((self.promoted_to.unwrap(), color));
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

                // make castle
                if self.is_castle {
                    if self.to.col == 2 && self.to.row == 0 {
                        new_board[0][3] = Some((Piece::Rook, Color::White));
                        new_board[0][0] = None;
                    } else if self.to.col == 6 && self.to.row == 0 {
                        new_board[0][5] = Some((Piece::Rook, Color::White));
                        new_board[0][7] = None;
                    } else if self.to.col == 2 && self.to.row == 7 {
                        new_board[7][3] = Some((Piece::Rook, Color::Black));
                        new_board[7][0] = None;
                    } else if self.to.col == 6 && self.to.row == 7 {
                        new_board[7][5] = Some((Piece::Rook, Color::Black));
                        new_board[7][7] = None;
                    }
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
        if self.promoted_to.is_some() {
            notation.push('=');
            notation.push_str(
                &self
                    .promoted_to
                    .unwrap()
                    .to_char()
                    .to_uppercase()
                    .to_string(),
            );
        }
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

    fn generate(depth: u8, position: &Position) -> (usize, Vec<Move>) {
        if depth == 0 {
            return (1, vec![]);
        }

        let moves = Move::legal_moves(&position);
        let mut count = 0;

        for move_ in &moves {
            let new_position = move_.make(&position);
            count += generate(depth - 1, &new_position).0;
        }

        (count, moves)
    }

    #[test]
    fn perft_6() {
        let new_position = || -> Position {
            Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
        };
        assert_eq!(generate(1, &new_position()).0, 46);
        //assert_eq!(generate(2, &new_position()).0, 2079);
        //assert_eq!(generate(3, &new_position()).0, 89890);
        //assert_eq!(generate(4, &new_position()).0, 3894594);
        //assert_eq!(generate(5, &new_position()).0, 164075551);
    }

    #[test]
    fn perft_5() {
        let new_position = || -> Position {
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
        };

        assert_eq!(generate(1, &new_position()).0, 44);
        assert_eq!(generate(2, &new_position()).0, 1486);
        assert_eq!(generate(3, &new_position()).0, 62379);
        //assert_eq!(generate(4, &new_position()).0, 2103487);
        //assert_eq!(generate(5, &new_position()).0, 89941194);
    }

    #[test]
    fn perft_4() {
        let new_position = || -> Position {
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
        };
        assert_eq!(generate(1, &new_position()).0, 6);
        assert_eq!(generate(2, &new_position()).0, 264);
        //assert_eq!(generate(3, &new_position()).0, 9467);
        //assert_eq!(generate(4, &new_position()).0, 422333);
        //assert_eq!(generate(5, &new_position()).0, 15833292);
    }

    #[test]
    fn perft_3() {
        let new_position =
            || -> Position { Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -") };

        assert_eq!(generate(1, &new_position()).0, 14);
        assert_eq!(generate(2, &new_position()).0, 191);
        //assert_eq!(generate(3, &new_position()).0, 2812);
        //assert_eq!(generate(4, &new_position()).0, 43238);
        //assert_eq!(generate(5, &new_position()).0, 674624);
    }

    #[test]
    fn perft_2() {
        let new_position = || -> Position {
            Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
        };

        assert_eq!(generate(1, &new_position()).0, 48);
        //assert_eq!(generate(2, &new_position()).0, 2039);

        //assert_eq!(generate(3, &new_position()).0, 97862);
        //assert_eq!(generate(4, &new_position()).0, 4085603);
        //assert_eq!(generate(5, &new_position()).0, 193690690);
    }

    #[test]
    fn perft_1() {
        assert_eq!(generate(2, &Position::new()).0, 400);
        assert_eq!(generate(3, &Position::new()).0, 8902);
        assert_eq!(generate(4, &Position::new()).0, 197281);
        //assert_eq!(generate(5, &Position::new()).0, 4865609);
        //assert_eq!(generate(6, &Position::new()).0, 119060324);
    }
}
