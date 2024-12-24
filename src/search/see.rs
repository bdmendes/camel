use crate::{
    core::{
        bitboard::Bitboard,
        moves::{gen::square_attackers, Move},
        piece::Piece,
        square::Square,
        Position,
    },
    evaluation::{Evaluable, ValueScore},
};

fn least_valuable(bb: Bitboard, position: &Position) -> Option<(Piece, Square)> {
    bb.into_iter()
        .map(|square| (position.piece_at(square).unwrap(), square))
        .min_by_key(|(piece, _)| piece.value())
}

pub fn see<const RETURN_EARLY: bool>(mov: Move, position: &Position) -> ValueScore {
    let (piece, color) = position.piece_color_at(mov.from()).unwrap();
    let their_piece = position.piece_at(mov.to()).unwrap_or(Piece::Pawn);

    // If we are only querying a positive SEE, we can return immediately
    // if we are capturing a more valuable piece.
    if RETURN_EARLY && (piece == Piece::Pawn || piece.value() <= their_piece.value()) {
        return 0;
    }

    // We need an auxiliary position to perform the search.
    // We also store a standing pat when it is our turn to move.
    let mut position = *position;
    let mut our_stand_pat = ValueScore::MIN;
    let mut their_stand_pat = ValueScore::MAX;

    // Make our move.
    let mut on_square = piece;
    let mut score = their_piece.value();
    let mut current_color = color.flipped();
    let mut current_sign = -1;
    position.clear_square(mov.from());

    loop {
        if current_color == color {
            our_stand_pat = our_stand_pat.max(score);
            if RETURN_EARLY && our_stand_pat >= 0 {
                return our_stand_pat;
            }
        } else {
            their_stand_pat = their_stand_pat.min(score);
            if RETURN_EARLY && their_stand_pat < 0 {
                return their_stand_pat;
            }
        }

        // We choose our least valuable piece to attack.
        let attackers = square_attackers(&position, mov.to(), current_color);

        if let Some((least_valuable_piece, attacker_square)) = least_valuable(attackers, &position)
        {
            // We capture the piece on the challenged square.
            score += current_sign * on_square.value();
            position.clear_square(attacker_square);

            // We put ourselves on the challenged square.
            on_square = least_valuable_piece;

            // Switch turns.
            current_color = current_color.flipped();
            current_sign = -current_sign;
        } else {
            // No more attackers.
            break;
        }
    }

    our_stand_pat.max(score).min(their_stand_pat)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        core::{piece::Piece, MoveStage, Position},
        evaluation::Evaluable,
    };

    #[test]
    fn see_1() {
        let position =
            Position::from_str("4r1kr/pp4pp/2pbRn2/q2P2B1/P7/2N5/1P3PP1/R2Q2K1 b - - 2 20")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c6d5").unwrap();
        assert_eq!(super::see::<false>(*mov, &position), Piece::Pawn.value());
        assert!(super::see::<true>(*mov, &position) >= 0);

        let mov = moves.iter().find(|mov| mov.to_string() == "e8e6").unwrap();
        assert_eq!(super::see::<false>(*mov, &position), 0);
        assert!(super::see::<true>(*mov, &position) >= 0);
    }

    #[test]
    fn see_2() {
        let position =
            Position::from_str("rnbqk2r/ppp1bppp/4pn2/3p4/2PP4/1QN5/PP2PPPP/R1B1KBNR w KQkq - 2 5")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c4d5").unwrap();
        assert_eq!(super::see::<false>(*mov, &position), 0);
        assert!(super::see::<true>(*mov, &position) >= 0);
    }

    #[test]
    fn see_3() {
        let position =
            Position::from_str("rnbqkb1r/1p1p1ppp/p3pn2/8/2PNP3/8/PP3PPP/RNBQKB1R w KQkq - 1 6")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "d4e6").unwrap();
        assert_eq!(
            super::see::<false>(*mov, &position),
            Piece::Pawn.value() - Piece::Knight.value()
        );
        assert!(super::see::<true>(*mov, &position) < 0);
    }

    #[test]
    fn see_4() {
        let position =
            Position::from_str("rnbqkb1r/1p1p1ppp/p3p3/8/2PNn3/2N5/PP3PPP/R1BQKB1R w KQkq - 0 7")
                .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "c3e4").unwrap();
        assert_eq!(super::see::<false>(*mov, &position), Piece::Knight.value());
        assert!(super::see::<true>(*mov, &position) >= 0);
    }

    #[test]
    fn see_5() {
        let position = Position::from_str(
            "r3r1k1/1pp1qpp1/p1nb1n2/3pNp1p/3PPB2/6QP/PPP2PP1/RN2R1K1 b - - 4 15",
        )
        .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "d6e5").unwrap();
        assert_eq!(
            super::see::<false>(*mov, &position),
            Piece::Pawn.value() - (Piece::Bishop.value() - Piece::Knight.value())
        );
        assert!(super::see::<true>(*mov, &position) >= 0);
    }

    #[test]
    fn see_6() {
        let position = Position::from_str(
            "rn2kbnr/ppp1pppp/1qb5/3p4/1P2P3/P4Q2/1BPP1PPP/RN2KBNR w KQkq - 3 6",
        )
        .unwrap();
        let moves = position.moves(MoveStage::All);

        let mov = moves.iter().find(|mov| mov.to_string() == "e4d5").unwrap();
        assert_eq!(super::see::<false>(*mov, &position), Piece::Pawn.value());
        assert!(super::see::<true>(*mov, &position) >= 0);
    }
}
