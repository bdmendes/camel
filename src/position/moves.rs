use std::fmt;

use super::{
    piece::{Color, Piece, DOWN, LEFT, RIGHT, UP},
    Position, Square, BOARD_SIZE, ROW_SIZE,
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
                } else {
                    break;
                }
            } else if to_piece.unwrap().color() != color {
                moves.push(Move::new(*square, Square { index: to_index }, true));
                break;
            } else {
                break;
            }
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
                    let can_castle_kingside = match color {
                        Color::White => position.castling_rights.white_kingside,
                        Color::Black => position.castling_rights.black_kingside,
                    };
                    if !can_castle_kingside {
                        continue;
                    }

                    if position.board[squares[3] as usize].is_none()
                        || position.board[squares[3] as usize].unwrap() != Piece::Rook(color)
                    {
                        continue;
                    }
                } else {
                    let can_castle_queenside = match color {
                        Color::White => position.castling_rights.white_queenside,
                        Color::Black => position.castling_rights.black_queenside,
                    };
                    if !can_castle_queenside {
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
    } else if let Some(piece) = position.board[move_.from.index as usize] {
        match piece {
            Piece::King(_) => match position.to_move {
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
    use std::collections::HashMap;

    use super::*;

    fn generate(
        original_depth: u8,
        current_depth: u8,
        position: &Position,
        memo: &mut HashMap<(String, u8), (usize, Vec<(Move, usize)>)>,
    ) -> (usize, Vec<(Move, usize)>) {
        if current_depth == 0 {
            return (1, vec![]);
        }

        let fen_hash = position.to_fen_hash();
        if let Some((count, moves)) = memo.get(&(fen_hash.to_owned(), current_depth)) {
            return (*count, moves.to_vec());
        }

        let moves = legal_moves(&position);
        let mut res = Vec::with_capacity(moves.len());
        let mut count = 0;

        for move_ in &moves {
            let new_position = make_move(&position, move_);
            let leaf_node_count =
                generate(original_depth, current_depth - 1, &new_position, memo).0;
            count += leaf_node_count;
            res.push((move_.to_owned(), leaf_node_count));
        }

        memo.insert(
            (fen_hash, current_depth),
            (
                count,
                if current_depth == original_depth {
                    res.to_vec()
                } else {
                    vec![]
                },
            ),
        );

        (count, res)
    }

    fn perft_divide(fen: &str, depth: u8, expected_nodes: Option<usize>) -> Vec<(Move, usize)> {
        let new_position = || -> Position { Position::from_fen(fen).unwrap() };

        let (count, moves) = generate(depth, depth, &new_position(), &mut HashMap::new());

        if expected_nodes.is_some() {
            assert_eq!(count, expected_nodes.unwrap());
        }

        moves
    }

    fn perft(fen: &str, depth: u8, expected_nodes: usize) {
        perft_divide(fen, depth, Some(expected_nodes));
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

    /* Expected divides taken from Stockfish */
    #[test]
    fn perft_kiwipete() {
        let kiwipete_test_fen =
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

        let expected_divides = [
            ("a2a3", 2186),
            ("b2b3", 1964),
            ("g2g3", 1882),
            ("d5d6", 1991),
            ("a2a4", 2149),
            ("g2g4", 1843),
            ("g2h3", 1970),
            ("d5e6", 2241),
            ("c3b1", 2038),
            ("c3d1", 2040),
            ("c3a4", 2203),
            ("c3b5", 2138),
            ("e5d3", 1803),
            ("e5c4", 1880),
            ("e5g4", 1878),
            ("e5c6", 2027),
            ("e5g6", 1997),
            ("e5d7", 2124),
            ("e5f7", 2080),
            ("d2c1", 1963),
            ("d2e3", 2136),
            ("d2f4", 2000),
            ("d2g5", 2134),
            ("d2h6", 2019),
            ("e2d1", 1733),
            ("e2f1", 2060),
            ("e2d3", 2050),
            ("e2c4", 2082),
            ("e2b5", 2057),
            ("e2a6", 1907),
            ("a1b1", 1969),
            ("a1c1", 1968),
            ("a1d1", 1885),
            ("h1f1", 1929),
            ("h1g1", 2013),
            ("f3d3", 2005),
            ("f3e3", 2174),
            ("f3g3", 2214),
            ("f3h3", 2360),
            ("f3f4", 2132),
            ("f3g4", 2169),
            ("f3f5", 2396),
            ("f3h5", 2267),
            ("f3f6", 2111),
            ("e1d1", 1894),
            ("e1f1", 1855),
            ("e1g1", 2059),
            ("e1c1", 1887),
        ];

        let moves = perft_divide(kiwipete_test_fen, 3, None);
        for (mv, count) in moves {
            if expected_divides.contains(&(&mv.to_string(), count - 1)) {
                continue;
            }
            println!("Unexpected divide: {} {}", mv, count);
        }
    }
}
