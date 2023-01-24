mod pvs;

use crate::{
    evaluate::{piece_value, psqt::psqt_value, Score, MATE_LOWER, MATE_UPPER},
    position::{
        moves::{position_is_check, Move},
        Piece, Position,
    },
};

use self::pvs::pvsearch;

#[allow(dead_code)]
pub struct Searcher {}

#[allow(dead_code)]
impl Searcher {
    pub fn new() -> Searcher {
        Searcher {}
    }

    fn is_quiet_move(move_: &Move, position: &Position) -> bool {
        let promotion = move_.promotion.is_some();
        let capture_piece = move_.capture
            && match position.at(move_.to).unwrap() {
                Piece::WP | Piece::BP => false,
                _ => true,
            };
        !promotion && !capture_piece
    }

    fn move_heuristic_value(move_: Move, position: &Position) -> Score {
        let mut score: Score = 0;

        if move_.promotion.is_some() {
            score += 4 * piece_value(move_.promotion.unwrap()); // usually ~3600 if queen
        }

        let moved_piece = position.at(move_.from).unwrap();

        if move_.capture {
            let moved_piece_value = piece_value(moved_piece);
            let captured_piece_value = piece_value(position.at(move_.to).unwrap());
            let value_diff = 2 * captured_piece_value - moved_piece_value; // if negative, we're losing material
            score += value_diff + piece_value(Piece::WQ); // [~1000, ~2800]
        }

        let start_psqt_value = psqt_value(moved_piece, move_.from, 0);
        let end_psqt_value = psqt_value(moved_piece, move_.to, 0);
        let psqt_value_diff = end_psqt_value - start_psqt_value;
        score += psqt_value_diff; // [~-200, ~200];

        score
    }

    fn game_over_evaluation(position: &Position, moves: &Vec<Move>) -> Option<Score> {
        // Flag 50 move rule draws
        if position.half_move_number >= 100 {
            return Some(0);
        }

        // Stalemate and checkmate detection
        if moves.len() == 0 {
            let is_check = position_is_check(position, position.to_move, None);
            return match is_check {
                true => Some(MATE_LOWER),
                false => Some(0),
            };
        }

        None
    }

    fn ids(&mut self, position: &Position, max_depth: u8) -> (Option<Move>, Score, usize) {
        // TODO: implement iterative deepening
        for i in 1..=max_depth {
            let (move_, eval, nodes) = pvsearch(self, position, i, MATE_LOWER, MATE_UPPER, i, 10);
            if i == max_depth {
                return (move_, eval, nodes);
            }
        }
        unreachable!()
    }

    pub fn search(&mut self, position: &Position, depth: u8) -> (Option<Move>, Score) {
        //let res = self.ids(position, depth);
        let res = pvsearch(self, position, depth, MATE_LOWER, MATE_UPPER, depth, 10);
        println!("searched: {}", res.2);
        (res.0, res.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::position::moves::legal_moves;

    use super::*;

    #[test]
    fn mvv_lva_psqt_heuristic_value() {
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        let mut moves = legal_moves(&position, position.to_move);
        moves.sort_by(|a, b| {
            Searcher::move_heuristic_value(*b, &position)
                .cmp(&Searcher::move_heuristic_value(*a, &position))
        });
        assert_eq!(moves[0].to_string(), "e2a6"); // equal trade of piece
        assert_eq!(moves[6].to_string(), "f3f6"); // queen for knight trade, after 2 pawn captures and 3 knight captures
    }

    #[test]
    fn search_xray_check() {
        let mut searcher = Searcher::new();
        let position = Position::from_fen("7R/7p/8/3pR1pk/pr1P4/5P2/P6r/3K4 w - - 0 35").unwrap();

        let depth = 2; // quiet search should increase depth due to capture on leaf node
        let (move_, _) = searcher.search(&position, depth);
        assert_eq!(move_.unwrap().to_string(), "h8h7");
    }

    #[test]
    fn search_double_attack() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("2kr3r/ppp2q2/4p2p/3nn3/2P3p1/1B5Q/P1P2PPP/R1B1K2R w KQ - 0 17")
                .unwrap();

        let depth = 2; // quiet search should increase depth due to capture on leaf node
        let (move_, _) = searcher.search(&position, depth);
        assert_eq!(move_.unwrap().to_string(), "h3g3");
    }

    #[test]
    fn search_mate_pattern() {
        let mut searcher = Searcher::new();
        let position =
            Position::from_fen("q5k1/3R2pp/p3pp2/N1b5/4b3/2B2r2/6PP/4QB1K b - - 5 35").unwrap();

        let depth = 2; // TODO: change to 5 after search is optimized
        let (move_, _) = searcher.search(&position, depth);
        assert_eq!(move_.unwrap().to_string(), "f3f2");
    }

    #[test]
    fn search_quickest_mate() {
        let mut searcher = Searcher::new();
        let position = Position::from_fen("8/k7/1pK5/8/8/8/5R2/8 w - - 0 1").unwrap();

        // Find quickest mate with exact ply and extra ply too
        for depth in [5, 6] {
            let (move_, eval) = searcher.search(&position, depth);
            assert_eq!(move_.unwrap().to_string(), "f2c2");
            assert!(eval >= MATE_UPPER - depth as Score);
        }
    }

    #[test]
    fn search_forced_capture() {
        let mut searcher = Searcher::new();
        let position = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        let depth = 2; // TODO: Increase to 4 after search is optimized
        let (move_, _) = searcher.search(&position, depth);
        assert_eq!(move_.unwrap().to_string(), "e2a6");
    }
}
