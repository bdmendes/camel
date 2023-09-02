use super::{
    attacks::{
        leapers::{KING_ATTACKS, KNIGHT_ATTACKS},
        magics::magic_index,
        magics::BISHOP_MAGICS,
        magics::ROOK_MAGICS,
        specials::generate_king_castles,
        specials::{generate_pawn_moves, pawn_attacks},
    },
    make_move, Move, MoveFlag, MoveVec,
};
use crate::position::{
    bitboard::Bitboard,
    board::{Board, Piece},
    square::Square,
    Color, Position,
};
use ahash::AHashMap;

pub struct MoveDirection;

impl MoveDirection {
    pub const NORTH: i8 = 8;
    pub const SOUTH: i8 = -8;
    pub const EAST: i8 = 1;
    pub const WEST: i8 = -1;

    pub const fn pawn_direction(color: Color) -> i8 {
        match color {
            Color::White => Self::NORTH,
            Color::Black => Self::SOUTH,
        }
    }
}

pub fn checked_by(board: &Board, color: Color) -> bool {
    let checked_king = board.pieces_bb(Piece::King) & board.occupancy_bb(color.opposite());
    if let Some(checked_king_square) = checked_king.into_iter().next() {
        square_attacked_by(board, checked_king_square, color)
    } else {
        false
    }
}

pub fn square_attacked_by(board: &Board, square: Square, color: Color) -> bool {
    // Attacked by pawn
    if pawn_attacks(board, color).is_set(square) {
        return true;
    }

    let occupancy_attacker = board.occupancy_bb(color);

    // Attacked by knight
    let attacker_knights = board.pieces_bb(Piece::Knight) & occupancy_attacker;
    let knight_attacks = KNIGHT_ATTACKS[square as usize];
    if (knight_attacks & attacker_knights).is_not_empty() {
        return true;
    }

    // Attacked by king
    let attacker_kings = board.pieces_bb(Piece::King) & occupancy_attacker;
    let king_attacks = KING_ATTACKS[square as usize];
    if (king_attacks & attacker_kings).is_not_empty() {
        return true;
    }

    let occupancy = board.occupancy_bb_all();

    // Attacked in file or rank
    let attacker_rooks_queens =
        (board.pieces_bb(Piece::Rook) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let rook_attacks = piece_attacks(Piece::Rook, square, occupancy);
    if (rook_attacks & attacker_rooks_queens).is_not_empty() {
        return true;
    }

    // Attacked in diagonal
    let attacker_bishops_queens =
        (board.pieces_bb(Piece::Bishop) | board.pieces_bb(Piece::Queen)) & occupancy_attacker;
    let bishop_attacks = piece_attacks(Piece::Bishop, square, occupancy);
    if (bishop_attacks & attacker_bishops_queens).is_not_empty() {
        return true;
    }

    false
}

pub fn piece_attacks(piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
    match piece {
        Piece::Knight => KNIGHT_ATTACKS[square as usize],
        Piece::King => KING_ATTACKS[square as usize],
        Piece::Rook => {
            let magic = &ROOK_MAGICS[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Bishop => {
            let magic = &BISHOP_MAGICS[square as usize];
            let index = magic_index(magic, occupancy);
            magic.attacks[index]
        }
        Piece::Queen => {
            piece_attacks(Piece::Rook, square, occupancy)
                | piece_attacks(Piece::Bishop, square, occupancy)
        }
        Piece::Pawn => unimplemented!("Pawn attacks are handled separately"),
    }
}

pub fn generate_regular_moves<const QUIESCE: bool>(
    board: &Board,
    piece: Piece,
    color: Color,
    moves: &mut MoveVec,
) {
    let occupancy_us = board.occupancy_bb(color);
    let pieces = board.pieces_bb(piece) & occupancy_us;

    if pieces.is_empty() {
        return;
    }

    let occupancy = board.occupancy_bb_all();
    let occupancy_them = board.occupancy_bb(color.opposite());

    for from_square in pieces {
        let attacks = piece_attacks(piece, from_square, occupancy) & !occupancy_us;

        for to_square in attacks {
            let flag = if occupancy_them.is_set(to_square) {
                MoveFlag::Capture
            } else {
                if QUIESCE {
                    continue;
                }
                MoveFlag::Quiet
            };

            moves.push(Move::new(from_square, to_square, flag));
        }
    }
}

pub fn generate_moves<const QUIESCE: bool, const PSEUDO: bool>(position: &Position) -> MoveVec {
    let mut moves = MoveVec::new();

    for piece in Piece::list().iter().filter(|p| **p != Piece::Pawn) {
        generate_regular_moves::<QUIESCE>(
            &position.board,
            *piece,
            position.side_to_move,
            &mut moves,
        );
    }

    generate_pawn_moves::<QUIESCE>(position, &mut moves);

    if !QUIESCE {
        generate_king_castles(position, &mut moves);
    }

    if !PSEUDO {
        moves.retain(|mov| match mov.flag() {
            MoveFlag::KingsideCastle | MoveFlag::QueensideCastle => true,
            _ => {
                let new_position = make_move::<false>(position, *mov);
                !checked_by(&new_position.board, new_position.side_to_move)
            }
        });
    }

    moves
}

type PerftResult = (u64, Vec<(Move, u64)>);

pub fn perft<const BULK_AT_HORIZON: bool, const HASH: bool, const SILENT: bool>(
    position: &Position,
    depth: u8,
) -> PerftResult {
    perft_internal::<true, BULK_AT_HORIZON, HASH, SILENT>(position, depth, &mut AHashMap::new())
}

fn perft_internal<
    const ROOT: bool,
    const BULK_AT_HORIZON: bool,
    const HASH: bool,
    const SILENT: bool,
>(
    position: &Position,
    depth: u8,
    cache: &mut AHashMap<(Position, u8), PerftResult>,
) -> PerftResult {
    if depth == 0 {
        return (1, vec![]);
    }

    if HASH {
        if let Some(res) = cache.get(&(*position, depth)) {
            return res.clone();
        }
    }

    let moves = generate_moves::<false, false>(position);

    if BULK_AT_HORIZON && depth == 1 {
        return (moves.len() as u64, vec![]);
    }

    let mut nodes = 0;
    let mut res = if ROOT { Vec::with_capacity(moves.len()) } else { Vec::new() };

    for mov in moves {
        let new_position = make_move::<true>(position, mov);
        let (count, _) =
            perft_internal::<false, BULK_AT_HORIZON, HASH, true>(&new_position, depth - 1, cache);
        nodes += count;

        if ROOT {
            res.push((mov, count));
        }

        if !SILENT {
            println!("{}: {}", mov, count);
        }
    }

    if HASH {
        cache.insert((*position, depth), (nodes, res.clone()));
    }

    (nodes, res)
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::gen::perft,
        position::{fen::KIWIPETE_WHITE_FEN, Color, Position},
    };

    #[test]
    fn square_is_attacked_1() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();

        assert!(super::square_attacked_by(&position.board, super::Square::E4, Color::Black));
        assert!(super::square_attacked_by(&position.board, super::Square::G2, Color::Black));
        assert!(super::square_attacked_by(&position.board, super::Square::A6, Color::White));
        assert!(!super::square_attacked_by(&position.board, super::Square::C7, Color::White));
        assert!(!super::square_attacked_by(&position.board, super::Square::B4, Color::White));
    }

    #[test]
    fn square_is_attacked_2() {
        let position =
            Position::from_fen("r3kbnr/pP3ppp/n3p3/q2pN2b/8/2N5/PPP1PP1P/R1BQKB1R b KQkq - 0 1")
                .unwrap();
        assert!(super::square_attacked_by(&position.board, super::Square::C8, Color::White));
    }

    #[test]
    fn gen_kiwipete_pseudo_regular() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let moves = super::generate_moves::<false, true>(&position);

        assert_eq!(moves.len(), 48);
    }

    #[test]
    fn gen_kiwipete_pseudo_quiesce() {
        let position = Position::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let moves = super::generate_moves::<true, true>(&position);

        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn gen_legals_simple() {
        let position = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

        let pseudo_moves = super::generate_moves::<false, true>(&position);
        assert_eq!(pseudo_moves.len(), 16);

        let legal_moves = super::generate_moves::<false, false>(&position);
        assert_eq!(legal_moves.len(), 14);
    }

    #[test]
    fn gen_legals_in_check() {
        let position =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap();

        let moves = super::generate_moves::<false, false>(&position);
        assert_eq!(moves.len(), 6);
    }

    fn expect_perft(fen: &str, depth: u8, nodes: u64) {
        let position = Position::from_fen(fen).unwrap();

        let time = std::time::Instant::now();
        let (res, _) = perft::<true, true, false>(&position, depth);
        let elapsed = time.elapsed().as_millis();

        println!(
            "\nDepth {}: {} in {} ms [{:.3} Mnps]",
            depth,
            res,
            elapsed,
            res as f64 / 1000.0 / (elapsed + 1) as f64
        );

        assert_eq!(res, nodes);
    }

    #[test]
    fn perft_gh_1() {
        expect_perft("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 1, 8);
    }

    #[test]
    fn perft_gh_2() {
        expect_perft("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3", 1, 8);
    }

    #[test]
    fn perft_gh_3() {
        expect_perft("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2", 1, 19);
    }

    #[test]
    fn perft_gh_4() {
        expect_perft("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2", 1, 44);
    }

    #[test]
    fn perft_gh_5() {
        expect_perft("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2", 1, 44);
    }

    #[test]
    fn perft_gh_6() {
        expect_perft("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9", 1, 39);
    }

    #[test]
    fn perft_gh_7() {
        expect_perft("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9);
    }

    #[test]
    fn perft_gh_8() {
        expect_perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379);
    }

    #[test]
    fn perft_gh_9() {
        expect_perft(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            3,
            89890,
        );
    }

    #[test]
    fn perft_gh_10() {
        expect_perft("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888);
    }

    #[test]
    fn perft_gh_11() {
        expect_perft("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133);
    }

    #[test]
    fn perft_gh_12() {
        expect_perft("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467);
    }

    #[test]
    fn perft_gh_13() {
        expect_perft("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072);
    }

    #[test]
    fn perft_gh_14() {
        expect_perft("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711);
    }

    #[test]
    fn perft_gh_15() {
        expect_perft("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206);
    }

    #[test]
    fn perft_gh_16() {
        expect_perft("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476);
    }

    #[test]
    fn perft_gh_17() {
        expect_perft("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001);
    }

    #[test]
    fn perft_gh_18() {
        expect_perft("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658);
    }

    #[test]
    fn perft_gh_19() {
        expect_perft("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342);
    }

    #[test]
    fn perft_gh_20() {
        expect_perft("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683);
    }

    #[test]
    fn perft_gh_21() {
        expect_perft("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217);
    }

    #[test]
    fn perft_gh_22() {
        expect_perft("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584);
    }

    #[test]
    fn perft_gh_23() {
        expect_perft("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527);
    }
}
