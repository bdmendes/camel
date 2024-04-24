use crate::{
    evaluation::{Evaluable, ValueScore},
    moves::{gen::square_attackers, Move},
    position::{
        bitboard::Bitboard,
        board::{Board, Piece},
        square::Square,
    },
};

fn least_valuable(bb: Bitboard, board: &Board) -> Option<(Piece, Square)> {
    bb.into_iter()
        .map(|square| (board.piece_at(square).unwrap(), square))
        .min_by_key(|(piece, _)| piece.value())
}

pub fn see<const RETURN_EARLY: bool>(mov: Move, board: &Board) -> ValueScore {
    let (piece, color) = board.piece_color_at(mov.from()).unwrap();
    let their_piece = board.piece_at(mov.to()).unwrap_or(Piece::Pawn);

    // If we are only querying a positive SEE, we can return immediately
    // if we are capturing a more valuable piece.
    if RETURN_EARLY && (piece == Piece::Pawn || piece.value() <= their_piece.value()) {
        return 0;
    }

    // We need an auxiliary board to perform the search.
    // We also store the max score when it is our turn.
    let mut board = *board;
    let mut max_score = ValueScore::MIN;

    // Make our move.
    let mut on_square = piece;
    let mut score = their_piece.value();
    let mut current_color = color.opposite();
    let mut current_sign = -1;
    board.clear_square(mov.from());

    loop {
        if current_color == color {
            if RETURN_EARLY && score >= 0 {
                return score;
            }
            max_score = max_score.max(score);
        } else if RETURN_EARLY && score < 0 {
            return score;
        }

        // We choose our least valuable piece to attack.
        let attackers = square_attackers::<false>(&board, mov.to(), current_color);

        if let Some((least_valuable_piece, attacker_square)) = least_valuable(attackers, &board) {
            // We capture the piece on the challenged square.
            score += current_sign * on_square.value();
            board.clear_square(attacker_square);

            // We put ourselves on the challenged square.
            on_square = least_valuable_piece;

            // Switch turns.
            current_color = current_color.opposite();
            current_sign = -current_sign;
        } else {
            // No more attackers.
            break;
        }
    }

    max_score.max(score)
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::Evaluable,
        moves::gen::MoveStage,
        position::{board::Piece, fen::FromFen, Position},
    };

    #[test]
    fn see_1() {
        let position =
            Position::from_fen("4r1kr/pp4pp/2pbRn2/q2P2B1/P7/2N5/1P3PP1/R2Q2K1 b - - 2 20")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c6d5").unwrap();
        assert_eq!(super::see::<false>(*mov, &position.board), Piece::Queen.value());
        assert!(super::see::<true>(*mov, &position.board) >= 0);

        let mov = moves.iter().find(|mov| mov.to_string() == "e8e6").unwrap();
        assert_eq!(super::see::<false>(*mov, &position.board), 0);
        assert!(super::see::<true>(*mov, &position.board) >= 0);
    }

    #[test]
    fn see_2() {
        let position =
            Position::from_fen("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/1QN5/PP2PPPP/R1B1KBNR w KQkq - 2 5")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c4d5").unwrap();
        assert_eq!(super::see::<false>(*mov, &position.board), 0);
        assert!(super::see::<true>(*mov, &position.board) >= 0);
    }

    #[test]
    fn see_3() {
        let position =
            Position::from_fen("rnbqkb1r/1p1p1ppp/p3pn2/8/2PNP3/8/PP3PPP/RNBQKB1R w KQkq - 1 6")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "d4e6").unwrap();
        assert_eq!(
            super::see::<false>(*mov, &position.board),
            Piece::Pawn.value() - Piece::Knight.value()
        );
        assert!(super::see::<true>(*mov, &position.board) < 0);
    }

    #[test]
    fn see_4() {
        let position =
            Position::from_fen("rnbqkb1r/1p1p1ppp/p3p3/8/2PNn3/2N5/PP3PPP/R1BQKB1R w KQkq - 0 7")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c3e4").unwrap();
        assert_eq!(super::see::<false>(*mov, &position.board), Piece::Knight.value());
        assert!(super::see::<true>(*mov, &position.board) >= 0);
    }
}
