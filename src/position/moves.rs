use super::{
    piece::{Color, Piece, DOWN, UP},
    CastlingRights, Position, PositionInfo, Square, BOARD_SIZE, ROW_SIZE,
};
use bitflags::bitflags;
use std::fmt;

bitflags! {
    pub struct MoveFlags: u8 {
        const CAPTURE = 0b001;
        const ENPASSANT = 0b010;
        const CASTLE = 0b100;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
                Some(promotion) =>
                    promotion.to_char().to_string().to_lowercase(),
                None => "".to_owned(),
            }
        )
    }
}

impl Move {
    pub fn new(from: Square, to: Square, flags: MoveFlags) -> Move {
        Move { from, to, flags, promotion: None }
    }

    pub fn is_tactical(&self) -> bool {
        self.flags.contains(MoveFlags::CAPTURE) || self.promotion.is_none()
    }
}

fn generate_regular_moves_from_square(
    position: &Position,
    square: Square,
    directions: &[isize],
    only_captures: bool,
) -> Vec<Move> {
    let piece = position.at(square).unwrap();
    let color = piece.color();
    let slides = piece.is_sliding();
    let mut moves = Vec::with_capacity(if slides && !only_captures {
        directions.len() * 4
    } else {
        directions.len()
    });

    for offset in directions {
        let mut current_offset = *offset;
        let mut last_column = square.col() as isize;

        loop {
            let to_index = (square.index as isize + current_offset) as usize;
            let current_col = (to_index % ROW_SIZE) as isize;
            let out_of_bounds =
                to_index >= BOARD_SIZE || (current_col - last_column).abs() > 2;
            if out_of_bounds {
                break;
            }

            let to_piece = position.board[to_index];
            if to_piece.is_none() {
                if !only_captures {
                    moves.push(Move::new(
                        square,
                        Square { index: to_index },
                        MoveFlags::empty(),
                    ));
                }
                if slides {
                    last_column = current_col;
                    current_offset += offset;
                    continue;
                }
            } else if to_piece.unwrap().color() != color {
                moves.push(Move::new(
                    square,
                    Square { index: to_index },
                    MoveFlags::CAPTURE,
                ));
            }
            break;
        }
    }
    moves
}

fn pseudo_legal_moves_from_square(
    position: &Position,
    square: Square,
    only_tactical: bool,
) -> Vec<Move> {
    let piece = position.at(square).unwrap();
    let color = piece.color();
    match piece {
        Piece::WP | Piece::BP => {
            let mut moves = generate_regular_moves_from_square(
                position,
                square,
                piece.unchecked_directions(),
                only_tactical,
            );

            // Do not advance if there is a piece in front; do not capture if there is no piece
            moves.retain(|move_| {
                let index_diff =
                    (move_.to.index as isize - move_.from.index as isize).abs();
                if index_diff == UP {
                    !move_.flags.contains(MoveFlags::CAPTURE)
                } else if index_diff == UP + UP {
                    let row = move_.from.row();
                    let can_advance_two = ((row == 1 && color == Color::White)
                        || (row == 6 && color == Color::Black))
                        && !move_.flags.contains(MoveFlags::CAPTURE);
                    let jumped_piece = position.board[(move_.from.index
                        as isize
                        + (if color == Color::White { UP } else { DOWN }))
                        as usize]
                        .is_some();
                    can_advance_two && !jumped_piece
                } else {
                    move_.flags.contains(MoveFlags::CAPTURE)
                        || match position.info.en_passant_square {
                            Some(en_passant_square) => {
                                move_.to == en_passant_square
                            }
                            None => false,
                        }
                }
            });

            // Add promotion and en passant
            let curr_moves_len = moves.len();
            for i in 0..curr_moves_len {
                let mut move_ = &mut moves[i];
                let row = move_.to.row();
                if row == 0 || row == 7 {
                    let mut under_promotion_moves =
                        Vec::<Move>::with_capacity(3);
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
                } else if let Some(en_passant_square) =
                    position.info.en_passant_square
                {
                    if move_.to == en_passant_square {
                        move_.flags |= MoveFlags::ENPASSANT;
                    }
                }
            }

            moves
        }
        Piece::WK | Piece::BK => {
            let mut moves = generate_regular_moves_from_square(
                position,
                square,
                piece.unchecked_directions(),
                only_tactical,
            );

            if only_tactical {
                return moves;
            }

            // Handle castling
            const CASTLE_SQUARES: [[usize; 5]; 4] = [
                [4, 5, 6, 7, BOARD_SIZE],     // White kingside
                [4, 3, 2, 1, 0],              // White queenside
                [60, 61, 62, 63, BOARD_SIZE], // Black kingside
                [60, 59, 58, 57, 56],         // Black queenside
            ];
            for i in 0..4 {
                // Check castle rights
                let can_castle = match i {
                    0 => position
                        .info
                        .castling_rights
                        .contains(CastlingRights::WHITE_KINGSIDE),
                    1 => position
                        .info
                        .castling_rights
                        .contains(CastlingRights::WHITE_QUEENSIDE),
                    2 => position
                        .info
                        .castling_rights
                        .contains(CastlingRights::BLACK_KINGSIDE),
                    3 => position
                        .info
                        .castling_rights
                        .contains(CastlingRights::BLACK_QUEENSIDE),
                    _ => unreachable!(),
                };
                if !can_castle {
                    continue;
                }

                // Check if the squares between the king and the rook are empty
                let squares = CASTLE_SQUARES[i];
                if squares[0] != square.index {
                    continue;
                }
                if position.board[squares[1]].is_some()
                    || position.board[squares[2]].is_some()
                {
                    continue;
                }
                let same_color_rook =
                    if color == Color::White { Piece::WR } else { Piece::BR };
                let kingside = i == 0 || i == 2;
                if !kingside {
                    if let Some(_) = position.board[squares[3]] {
                        continue;
                    }
                }
                if let Some(to_piece) =
                    position.board[squares[3 + if kingside { 0 } else { 1 }]]
                {
                    if to_piece != same_color_rook {
                        continue;
                    }
                } else {
                    continue;
                }

                moves.push(Move {
                    from: square,
                    to: Square { index: squares[2] },
                    flags: MoveFlags::CASTLE,
                    promotion: None,
                });
            }

            moves
        }
        _ => generate_regular_moves_from_square(
            position,
            square,
            piece.unchecked_directions(),
            only_tactical,
        ),
    }
}

pub fn pseudo_legal_moves(
    position: &Position,
    to_move: Color,
    only_tactical: bool,
) -> Vec<Move> {
    let mut moves = Vec::with_capacity(40);
    for index in 0..BOARD_SIZE {
        let piece = position.board[index];
        if piece.is_none() || piece.unwrap().color() != to_move {
            continue;
        }
        moves.extend(pseudo_legal_moves_from_square(
            position,
            Square { index },
            only_tactical,
        ));
    }
    moves
}

pub fn legal_moves(
    position: &Position,
    to_move: Color,
    only_non_quiet: bool,
) -> Vec<Move> {
    let mut moves = pseudo_legal_moves(position, to_move, only_non_quiet);

    moves.retain(|move_| {
        let castle_passent_squares = if move_.flags.contains(MoveFlags::CASTLE)
        {
            Some([
                move_.from,
                Square { index: (move_.to.index + move_.from.index) / 2 },
            ])
        } else {
            None
        };

        // Do not allow moves that leave the player in check
        let new_position = make_move(position, *move_);
        !position_is_check(&new_position, to_move, castle_passent_squares)
    });

    moves
}

pub fn position_is_check(
    position: &Position,
    checked_player: Color,
    castle_passent_squares: Option<[Square; 2]>,
) -> bool {
    let opposing_color = checked_player.opposite();
    let opponent_moves = pseudo_legal_moves(
        position,
        opposing_color,
        castle_passent_squares.is_none(),
    );

    for move_ in opponent_moves {
        if let Some(piece) = position.at(move_.to) {
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
    }

    false
}

pub fn make_move(position: &Position, move_: Move) -> Position {
    let mut new_board = position.board;
    new_board[move_.to.index] = new_board[move_.from.index];
    new_board[move_.from.index] = None;

    // En passant
    if move_.flags.contains(MoveFlags::ENPASSANT) {
        let capture_square = match position.info.to_move {
            Color::White => move_.to.index as isize + DOWN,
            Color::Black => move_.to.index as isize + UP,
        };
        new_board[capture_square as usize] = None;
    }

    // Promotion
    if let Some(promotion_piece) = move_.promotion {
        new_board[move_.to.index] = Some(promotion_piece);
    }

    // Castling
    let mut new_castling_rights = position.info.castling_rights;
    let piece = position.at(move_.from).unwrap();
    if move_.flags.contains(MoveFlags::CASTLE) {
        if piece == Piece::WK {
            new_castling_rights &= !(CastlingRights::WHITE_KINGSIDE
                | CastlingRights::WHITE_QUEENSIDE);
            if move_.to.index == 6 {
                new_board[5] = new_board[7];
                new_board[7] = None;
            } else if move_.to.index == 2 {
                new_board[3] = new_board[0];
                new_board[0] = None;
            }
        } else if piece == Piece::BK {
            new_castling_rights &= !(CastlingRights::BLACK_KINGSIDE
                | CastlingRights::BLACK_QUEENSIDE);
            if move_.to.index == 62 {
                new_board[61] = new_board[63];
                new_board[63] = None;
            } else if move_.to.index == 58 {
                new_board[59] = new_board[56];
                new_board[56] = None;
            }
        }
    } else {
        match piece {
            Piece::WK => {
                new_castling_rights &= !(CastlingRights::WHITE_KINGSIDE
                    | CastlingRights::WHITE_QUEENSIDE);
            }
            Piece::BK => {
                new_castling_rights &= !(CastlingRights::BLACK_KINGSIDE
                    | CastlingRights::BLACK_QUEENSIDE);
            }
            Piece::WR => {
                if move_.from.index == 0 {
                    new_castling_rights &= !CastlingRights::WHITE_QUEENSIDE;
                } else if move_.from.index == 7 {
                    new_castling_rights &= !CastlingRights::WHITE_KINGSIDE;
                }
            }
            Piece::BR => {
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
        info: PositionInfo {
            to_move: position.info.to_move.opposite(),
            castling_rights: new_castling_rights,
            en_passant_square: match position.at(move_.from).unwrap() {
                Piece::WP | Piece::BP => {
                    if (move_.to.index as isize - move_.from.index as isize)
                        .abs()
                        == 2 * UP
                    {
                        Some(Square {
                            index: (move_.to.index + move_.from.index) / 2,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            },
            half_move_number: if move_.flags.contains(MoveFlags::CAPTURE)
                || matches!(
                    position.at(move_.from).unwrap(),
                    Piece::WP | Piece::BP
                ) {
                0
            } else {
                position.info.half_move_number + 1
            },
            full_move_number: if position.info.to_move == Color::Black {
                position.info.full_move_number + 1
            } else {
                position.info.full_move_number
            },
        },
    }
}

pub fn make_null_move(position: &Position) -> Position {
    Position {
        board: position.board,
        info: PositionInfo {
            to_move: position.info.to_move.opposite(),
            castling_rights: position.info.castling_rights,
            en_passant_square: None,
            half_move_number: position.info.half_move_number,
            full_move_number: position.info.full_move_number,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::position::zobrist::ZobristHash;

    use super::*;

    #[test]
    fn perft_start() {
        let position = super::Position::new();
        let moves = legal_moves(&position, position.info.to_move, false);
        assert_eq!(moves.len(), 20);
    }

    fn generate(
        original_depth: u8,
        current_depth: u8,
        position: &Position,
        memo: &mut HashMap<(ZobristHash, u8), (usize, Vec<(Move, usize)>)>,
    ) -> (usize, Vec<(Move, usize)>) {
        if current_depth == 0 {
            return (1, vec![]);
        }

        let zobrist_hash = position.to_zobrist_hash();
        if let Some((count, moves)) = memo.get(&(zobrist_hash, current_depth)) {
            return (*count, moves.to_vec());
        }

        let moves = legal_moves(&position, position.info.to_move, false);
        let mut res = Vec::with_capacity(moves.len());
        let mut count = 0;

        for move_ in &moves {
            let new_position = make_move(&position, *move_);
            let leaf_node_count = generate(
                original_depth,
                current_depth - 1,
                &new_position,
                memo,
            )
            .0;
            count += leaf_node_count;
            res.push((move_.to_owned(), leaf_node_count));
        }

        memo.insert(
            (zobrist_hash, current_depth),
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

    fn perft_divide(
        fen: &str,
        depth: u8,
        expected_nodes: Option<usize>,
    ) -> Vec<(Move, usize)> {
        let new_position = || -> Position { Position::from_fen(fen).unwrap() };

        let intial_time = std::time::Instant::now();

        let (count, moves) =
            generate(depth, depth, &new_position(), &mut HashMap::new());

        let elapsed = intial_time.elapsed();

        println!(
            "Generated {} nodes in {}.{:.3} seconds ({:.3} nodes/s)",
            count,
            elapsed.as_secs(),
            elapsed.subsec_millis(),
            count as f64
                / (elapsed.as_secs() as f64
                    + elapsed.subsec_millis() as f64 / 1000.0)
        );

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
    fn perft_gh_1() {
        perft("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8);
    }

    #[test]
    fn perft_gh_2() {
        perft("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8);
    }

    #[test]
    fn perft_gh_3() {
        perft(
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2",
            1,
            19,
        );
    }

    #[test]
    fn perft_gh_4() {
        perft(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
            1,
            44,
        );
    }

    #[test]
    fn perft_gh_5() {
        perft(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
            1,
            44,
        );
    }

    #[test]
    fn perft_gh_6() {
        perft(
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9",
            1,
            39,
        );
    }

    #[test]
    fn perft_gh_7() {
        perft("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9);
    }

    #[test]
    fn perft_gh_8() {
        perft(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            3,
            62379,
        );
    }

    #[test]
    fn perft_gh_9() {
        perft("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 3, 89890);
    }

    #[test]
    fn perft_gh_10() {
        perft("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888);
    }

    #[test]
    fn perft_gh_11() {
        perft("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133);
    }

    #[test]
    fn perft_gh_12() {
        perft("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467);
    }

    #[test]
    fn perft_gh_13() {
        perft("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072);
    }

    #[test]
    fn perft_gh_14() {
        perft("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711);
    }

    #[test]
    fn perft_gh_15() {
        perft("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206);
    }

    #[test]
    fn perft_gh_16() {
        perft("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476);
    }

    #[test]
    fn perft_gh_17() {
        perft("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001);
    }

    #[test]
    fn perft_gh_18() {
        perft("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658);
    }

    #[test]
    fn perft_gh_19() {
        perft("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342);
    }

    #[test]
    fn perft_gh_20() {
        perft("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683);
    }

    #[test]
    fn perft_gh_21() {
        perft("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217);
    }

    #[test]
    fn perft_gh_22() {
        perft("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584);
    }

    #[test]
    fn perft_gh_23() {
        perft("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527);
    }
}
