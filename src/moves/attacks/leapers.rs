use crate::{moves::gen::MoveDirection, position::bitboard::Bitboard};

pub type LeaperAttackMap = [Bitboard; 64];

pub static KNIGHT_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    MoveDirection::NORTH + 2 * MoveDirection::WEST,
    MoveDirection::NORTH + 2 * MoveDirection::EAST,
    MoveDirection::SOUTH + 2 * MoveDirection::WEST,
    MoveDirection::SOUTH + 2 * MoveDirection::EAST,
    2 * MoveDirection::NORTH + MoveDirection::WEST,
    2 * MoveDirection::NORTH + MoveDirection::EAST,
    2 * MoveDirection::SOUTH + MoveDirection::WEST,
    2 * MoveDirection::SOUTH + MoveDirection::EAST,
]);

pub static KING_ATTACKS: LeaperAttackMap = init_leaper_attacks(&[
    MoveDirection::NORTH,
    MoveDirection::NORTH + MoveDirection::EAST,
    MoveDirection::EAST,
    MoveDirection::SOUTH + MoveDirection::EAST,
    MoveDirection::SOUTH,
    MoveDirection::SOUTH + MoveDirection::WEST,
    MoveDirection::WEST,
    MoveDirection::NORTH + MoveDirection::WEST,
]);

const fn init_leaper_attacks(move_directions: &[i8]) -> LeaperAttackMap {
    let mut attacks: LeaperAttackMap = [Bitboard::new(0); 64];

    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let mut bb = 0;

        let mut i = 0;
        while i < move_directions.len() {
            let target_square = square + move_directions[i];
            let target_square_file = target_square % 8;
            if target_square >= 0 && target_square < 64 && (target_square_file - file).abs() <= 2 {
                bb |= 1 << target_square;
            }
            i += 1;
        }

        attacks[square as usize] = Bitboard::new(bb);
        square += 1
    }

    attacks
}

#[cfg(test)]
mod tests {
    use crate::{
        moves::{gen::generate_regular_moves, Move, MoveFlag},
        position::{
            board::{Board, Piece},
            fen::KIWIPETE_WHITE_FEN,
            square::Square,
            Color,
        },
    };

    fn generate_knight_moves<const QUIESCE: bool>(
        board: &Board,
        color: Color,
        moves: &mut Vec<Move>,
    ) {
        generate_regular_moves::<QUIESCE>(board, Piece::Knight, color, moves);
    }

    #[test]
    fn center_clean_knight() {
        let mut board = Board::default();
        let us_color = Color::White;

        board.set_square(Square::E4, Piece::Knight, us_color);

        let mut moves = Vec::new();
        generate_knight_moves::<false>(&board, us_color, &mut moves);

        assert_eq!(moves.len(), 8);

        let expected_moves = vec![
            Move::new(Square::E4, Square::F2, MoveFlag::Quiet),
            Move::new(Square::E4, Square::G3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::G5, MoveFlag::Quiet),
            Move::new(Square::E4, Square::F6, MoveFlag::Quiet),
            Move::new(Square::E4, Square::D6, MoveFlag::Quiet),
            Move::new(Square::E4, Square::C5, MoveFlag::Quiet),
            Move::new(Square::E4, Square::C3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::D2, MoveFlag::Quiet),
        ];

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn top_left_knight() {
        let mut board = Board::default();
        let us_color = Color::White;

        board.set_square(Square::A8, Piece::Knight, us_color);

        let mut moves = Vec::new();
        generate_knight_moves::<false>(&board, us_color, &mut moves);

        let expected_moves = vec![
            Move::new(Square::A8, Square::B6, MoveFlag::Quiet),
            Move::new(Square::A8, Square::C7, MoveFlag::Quiet),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn kiwipete_regular_knight() {
        let board = Board::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let us_color = Color::White;

        let mut moves = Vec::new();
        generate_knight_moves::<false>(&board, us_color, &mut moves);

        let expected_moves = [
            Move::new(Square::E5, Square::G4, MoveFlag::Quiet),
            Move::new(Square::E5, Square::G6, MoveFlag::Capture),
            Move::new(Square::E5, Square::F7, MoveFlag::Capture),
            Move::new(Square::E5, Square::D7, MoveFlag::Capture),
            Move::new(Square::E5, Square::C6, MoveFlag::Quiet),
            Move::new(Square::E5, Square::C4, MoveFlag::Quiet),
            Move::new(Square::E5, Square::D3, MoveFlag::Quiet),
            Move::new(Square::C3, Square::B1, MoveFlag::Quiet),
            Move::new(Square::C3, Square::A4, MoveFlag::Quiet),
            Move::new(Square::C3, Square::B5, MoveFlag::Quiet),
            Move::new(Square::C3, Square::D1, MoveFlag::Quiet),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }

    #[test]
    fn kiwipete_quiesce_knight() {
        let board = Board::from_fen(KIWIPETE_WHITE_FEN).unwrap();
        let us_color = Color::White;

        let mut moves = Vec::new();
        generate_knight_moves::<true>(&board, us_color, &mut moves);

        let expected_moves = [
            Move::new(Square::E5, Square::G6, MoveFlag::Capture),
            Move::new(Square::E5, Square::F7, MoveFlag::Capture),
            Move::new(Square::E5, Square::D7, MoveFlag::Capture),
        ];

        assert_eq!(moves.len(), expected_moves.len());

        for expected_move in expected_moves {
            assert!(moves.contains(&expected_move));
        }
    }
}
