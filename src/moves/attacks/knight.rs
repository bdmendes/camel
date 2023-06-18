use crate::{
    moves::gen::{AttackMap, MoveDirection},
    position::bitboard::Bitboard,
};

pub const KNIGHT_ATTACKS: AttackMap = init_knight_attacks();

const fn init_knight_attacks() -> AttackMap {
    let mut attacks: AttackMap = [Bitboard::new(0); 64];

    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bb = 0;

        if file >= 1 && rank >= 2 {
            bb |= 1 << (square + MoveDirection::SOUTH * 2 + MoveDirection::WEST);
        }

        if file >= 2 && rank >= 1 {
            bb |= 1 << (square + MoveDirection::SOUTH + MoveDirection::WEST * 2);
        }

        if file <= 5 && rank >= 1 {
            bb |= 1 << (square + MoveDirection::SOUTH + MoveDirection::EAST * 2);
        }

        if file <= 6 && rank >= 2 {
            bb |= 1 << (square + MoveDirection::SOUTH * 2 + MoveDirection::EAST);
        }

        if file <= 6 && rank <= 5 {
            bb |= 1 << (square + MoveDirection::NORTH * 2 + MoveDirection::EAST);
        }

        if file <= 5 && rank <= 6 {
            bb |= 1 << (square + MoveDirection::NORTH + MoveDirection::EAST * 2);
        }

        if file >= 2 && rank <= 6 {
            bb |= 1 << (square + MoveDirection::NORTH + MoveDirection::WEST * 2);
        }

        if file >= 1 && rank <= 5 {
            bb |= 1 << (square + MoveDirection::NORTH * 2 + MoveDirection::WEST);
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
            fen::KIWIPETE_FEN,
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
    fn center_clean() {
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
    fn top_left() {
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
    fn kiwipete_regular() {
        let board = Board::from_fen(KIWIPETE_FEN).unwrap();
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
    fn kiwipete_quiesce() {
        let board = Board::from_fen(KIWIPETE_FEN).unwrap();
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
