use std::fmt;

use super::{
    piece::{Color, Piece, LEFT, RIGHT, UP},
    Position, Square, BOARD_SIZE, ROW_SIZE,
};

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

fn pseudo_legal_from_square(position: &Position, square: &Square) -> Vec<Move> {
    let regular_moves = |color: Color, crawls: bool, directions: Vec<i8>, square: &Square| {
        let mut moves = Vec::new();

        for offset in &directions {
            let mut current_offset = *offset;
            let mut last_column = square.col(); // for out of bounds check
            loop {
                let to_index = (square.index as i8 + current_offset) as u8;
                if to_index >= BOARD_SIZE {
                    break;
                }
                let current_col = to_index % ROW_SIZE;
                if (current_col as i8 - last_column as i8).abs() > 2 {
                    break;
                }

                let to_piece = position.board[to_index as usize];
                if to_piece.is_none() {
                    moves.push(Move {
                        from: *square,
                        to: Square {
                            index: to_index as u8,
                        },
                        capture: false,
                        promotion: None,
                        enpassant: false,
                        castle: false,
                    });
                    if crawls {
                        last_column = current_col;
                        current_offset += offset;
                        continue;
                    }
                } else if to_piece.unwrap().color() != color {
                    moves.push(Move {
                        from: *square,
                        to: Square {
                            index: to_index as u8,
                        },
                        capture: true,
                        promotion: None,
                        enpassant: false,
                        castle: false,
                    });
                }

                break;
            }
        }
        moves
    };

    let piece = position.at(square).unwrap();
    match piece {
        Piece::Bishop(color) | Piece::Knight(color) | Piece::Rook(color) | Piece::Queen(color) => {
            regular_moves(
                color,
                piece.is_crawling(),
                piece.unchecked_directions(),
                square,
            )
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
                (front_direction + LEFT) as u8,
                (front_direction + RIGHT) as u8,
            ] {
                let capture_square = (square.index as i8 + capture_direction as i8) as u8;
                if capture_square > BOARD_SIZE {
                    break;
                }
                let to_piece = position.board[capture_square as usize];
                let is_opponent_piece = to_piece.is_some() && to_piece.unwrap().color() != color;
                let is_en_passant = position.en_passant_square.is_some()
                    && position.en_passant_square.unwrap().index == capture_square;
                if is_opponent_piece || is_en_passant {
                    directions.push(capture_direction as i8);
                }
            }

            let mut moves = regular_moves(color, false, directions, square);
            let mut under_promotion_moves = Vec::<Move>::new();
            for move_ in &mut moves {
                // Handle promotion and en passant
                if move_.to.row() == 0 || move_.to.row() == 7 {
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
            let mut moves = regular_moves(color, false, piece.unchecked_directions(), square);

            let mut castle_moves = Vec::<Move>::with_capacity(2);
            match color {
                Color::White => {
                    if position.castling_rights.white_kingside
                        && position.board[5].is_none()
                        && position.board[6].is_none()
                        && position.board[7].is_some()
                        && position.board[7].unwrap() == Piece::Rook(Color::White)
                    {
                        castle_moves.push(Move {
                            from: *square,
                            to: Square { index: 6 },
                            capture: false,
                            promotion: None,
                            enpassant: false,
                            castle: true,
                        });
                    }
                    if position.castling_rights.white_queenside
                        && position.board[1].is_none()
                        && position.board[2].is_none()
                        && position.board[3].is_none()
                        && position.board[0].is_some()
                        && position.board[0].unwrap() == Piece::Rook(Color::White)
                    {
                        castle_moves.push(Move {
                            from: *square,
                            to: Square { index: 2 },
                            capture: false,
                            promotion: None,
                            enpassant: false,
                            castle: true,
                        });
                    }
                }
                Color::Black => {
                    if position.castling_rights.black_kingside
                        && position.board[61].is_none()
                        && position.board[62].is_none()
                        && position.board[63].is_some()
                        && position.board[63].unwrap() == Piece::Rook(Color::Black)
                    {
                        castle_moves.push(Move {
                            from: *square,
                            to: Square { index: 62 },
                            capture: false,
                            promotion: None,
                            enpassant: false,
                            castle: true,
                        });
                    }
                    if position.castling_rights.black_queenside
                        && position.board[57].is_none()
                        && position.board[58].is_none()
                        && position.board[59].is_none()
                        && position.board[56].is_some()
                        && position.board[56].unwrap() == Piece::Rook(Color::Black)
                    {
                        castle_moves.push(Move {
                            from: *square,
                            to: Square { index: 58 },
                            capture: false,
                            promotion: None,
                            enpassant: false,
                            castle: true,
                        });
                    }
                }
            }
            moves.extend(castle_moves);

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
        moves.extend(pseudo_legal_from_square(position, &Square { index }));
    }
    moves
}

pub fn make_move(position: &Position, move_: &Move) -> Position {
    let mut new_board = position.board.clone();
    new_board[move_.to.index as usize] = new_board[move_.from.index as usize];
    new_board[move_.from.index as usize] = None;

    // En passant
    if move_.enpassant {
        let capture_square = match position.next_to_move {
            Color::White => move_.to.index - 8,
            Color::Black => move_.to.index + 8,
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
            if piece == Piece::King(position.next_to_move) {
                match position.next_to_move {
                    Color::White => {
                        new_castling_rights.white_kingside = false;
                        new_castling_rights.white_queenside = false;
                        if move_.to.index == 6 {
                            new_board[5] = new_board[7];
                            new_board[7] = None;
                        } else if move_.to.index == 2 {
                            new_board[3] = new_board[0];
                            new_board[0] = None;
                        }
                    }
                    Color::Black => {
                        new_castling_rights.black_kingside = false;
                        new_castling_rights.black_queenside = false;
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
    } else if let Some(piece) = new_board[move_.from.index as usize] {
        match piece {
            Piece::King(_) => match position.next_to_move {
                Color::White => {
                    new_castling_rights.white_kingside = false;
                    new_castling_rights.white_queenside = false;
                }
                Color::Black => {
                    new_castling_rights.black_kingside = false;
                    new_castling_rights.black_queenside = false;
                }
            },
            Piece::Rook(Color::White) => {
                if move_.from.index == 0 {
                    new_castling_rights.white_queenside = false;
                } else if move_.from.index == 7 {
                    new_castling_rights.white_kingside = false;
                }
            }
            Piece::Rook(Color::Black) => {
                if move_.from.index == 56 {
                    new_castling_rights.black_queenside = false;
                } else if move_.from.index == 63 {
                    new_castling_rights.black_kingside = false;
                }
            }
            _ => {}
        }
    }

    Position {
        board: new_board,
        next_to_move: position.next_to_move.opposing(),
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
        full_move_number: if position.next_to_move == Color::Black {
            position.full_move_number + 1
        } else {
            position.full_move_number
        },
    }
}

pub fn legal_moves(position: &Position) -> Vec<Move> {
    let mut moves = pseudo_legal_moves(position, position.next_to_move);
    moves.retain(|move_| {
        if let Some(piece) = position.at(&move_.from) {
            if piece == Piece::King(position.next_to_move) {
                if move_.castle && position.is_check(position.next_to_move) {
                    return false;
                }
            }
        }

        let new_position = make_move(position, move_);
        !new_position.is_check(position.next_to_move)
    });
    moves
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn generate(
        original_depth: u8,
        current_depth: u8,
        position: &Position,
        memo: &mut HashMap<(String, u8), (usize, Vec<Move>)>,
    ) -> (usize, Vec<Move>) {
        if current_depth == 0 {
            return (1, vec![]);
        }

        let fen_hash = position.to_fen_hash();
        if let Some((count, moves)) = memo.get(&(fen_hash.to_owned(), current_depth)) {
            return (*count, moves.to_vec());
        }

        let moves = legal_moves(&position);
        let mut count = 0;

        for move_ in &moves {
            let new_position = make_move(&position, move_);
            count += generate(original_depth, current_depth - 1, &new_position, memo).0;
        }

        memo.insert(
            (fen_hash, current_depth),
            (
                count,
                if current_depth == original_depth {
                    moves.to_vec()
                } else {
                    vec![]
                },
            ),
        );

        (count, moves)
    }

    fn perft(fen: &str, depth: u8, expected_nodes: usize) {
        let new_position = || -> Position { Position::from_fen(fen).unwrap() };

        let (count, moves) = generate(depth, depth, &new_position(), &mut HashMap::new());
        for move_ in moves {
            //println!("{}", move_);
        }
        assert_eq!(count, expected_nodes);
    }

    /* Taken from https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9 */
    #[test]
    fn gh_perft_1() {
        perft("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8);
    }

    #[test]
    fn gh_perft_2() {
        perft("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8);
    }

    #[test]
    fn gh_perft_3() {
        perft(
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2",
            1,
            19,
        );
    }

    #[test]
    fn gh_perft_4() {
        perft(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
            1,
            44,
        );
    }

    #[test]
    fn gh_perft_5() {
        perft(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
            1,
            44,
        );
    }

    #[test]
    fn gh_perft_6() {
        perft(
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9",
            1,
            39,
        );
    }

    #[test]
    fn gh_perft_7() {
        perft("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9);
    }

    #[test]
    fn gh_perft_8() {
        perft(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            3,
            62379,
        );
    }

    #[test]
    fn gh_perft_9() {
        perft(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            3,
            89890,
        );
    }

    #[test]
    fn gh_perft_10() {
        perft("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888);
    }

    #[test]
    fn gh_perft_11() {
        perft("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133);
    }

    #[test]
    fn gh_perft_12() {
        perft("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467);
    }

    #[test]
    fn gh_perft_13() {
        perft("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072);
    }

    #[test]
    fn gh_perft_14() {
        perft("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711);
    }

    #[test]
    fn gh_perft_15() {
        perft("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206);
    }

    #[test]
    fn gh_perft_16() {
        perft("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476);
    }

    #[test]
    fn gh_perft_17() {
        perft("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001);
    }

    #[test]
    fn gh_perft_18() {
        perft("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658);
    }

    #[test]
    fn gh_perft_19() {
        perft("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342);
    }

    #[test]
    fn gh_perft_20() {
        perft("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683);
    }

    #[test]
    fn gh_perft_21() {
        perft("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217);
    }

    #[test]
    fn gh_perft_22() {
        perft("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584);
    }

    #[test]
    fn gh_perft_23() {
        perft("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527);
    }

    /* Taken from http://www.rocechess.ch/perft.html */
    #[test]
    fn roce_perft_1() {
        perft(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            3,
            97862,
        );
    }

    #[test]
    fn roce_perft_2() {
        perft("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 4, 182838);
    }
}
