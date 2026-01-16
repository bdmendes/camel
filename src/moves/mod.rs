use crate::position::{Position, piece::Piece, square::Square};

use primitive_enum::primitive_enum;
use std::fmt::Display;

pub mod generate;
pub mod make;
pub mod see;

primitive_enum! { MoveFlag u8;
    Quiet,
    DoublePawnPush,
    KingsideCastle,
    QueensideCastle,
    Capture,
    EnpassantCapture,
    KnightPromotion = 8,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KnightPromotionCapture,
    BishopPromotionCapture,
    RookPromotionCapture,
    QueenPromotionCapture,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Move(u16);

impl Move {
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move((from as u16) | ((to as u16) << 6) | ((flag as u16) << 12))
    }

    pub fn from(&self) -> Square {
        Square::from((self.0 & 0x3F) as u8).unwrap()
    }

    pub fn to(&self) -> Square {
        Square::from(((self.0 & 0xFC0) >> 6) as u8).unwrap()
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from(((self.0 & 0xF000) >> 12) as u8).unwrap()
    }

    pub fn is_capture(&self) -> bool {
        ((1 << 14) & self.0) != 0
    }

    pub fn is_quiet(&self) -> bool {
        !self.is_capture() && self.promotion_piece().is_none()
    }

    pub fn is_reversible(&self, position: &Position) -> bool {
        self.is_quiet()
            && !matches!(self.flag(), MoveFlag::DoublePawnPush | MoveFlag::KingsideCastle | MoveFlag::QueensideCastle,)
            && position.piece_at(self.from()) != Some(Piece::Pawn)
    }

    pub fn promotion_piece(&self) -> Option<Piece> {
        if ((1 << 15) & self.0) == 0 {
            None
        } else {
            Some(match self.flag() {
                MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Piece::Knight,
                MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Piece::Bishop,
                MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Piece::Rook,
                _ => Piece::Queen,
            })
        }
    }

    pub fn pseudo_legal(&self, position: &Position) -> bool {
        position.color_at(self.from()) == Some(position.side_to_move())
            && (self.is_capture() && position.color_at(self.to()) != Some(position.side_to_move())
                || position.color_at(self.to()).is_none())
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.from(), self.to(), self.promotion_piece().map_or(String::new(), |p| p.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveFlag};
    use crate::moves::MoveFlag::*;
    use crate::position::fen::{Fen, KIWIPETE_POSITION, START_POSITION};
    use crate::position::piece::Piece::*;
    use crate::position::square::Square::*;
    use crate::position::{MoveStage, Position};
    use crate::position::{piece::Piece, square::Square};
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case(E4, H8, Quiet, true, false, true, None)]
    #[case(E2, E4, Quiet, true, false, false, None)]
    #[case(E2, E4, DoublePawnPush, true, false, false, None)]
    #[case(E1, G1, KingsideCastle, true, false, false, None)]
    #[case(E8, C8, QueensideCastle, true, false, false, None)]
    #[case(E4, E5, Capture, false, true, false, None)]
    #[case(D5, C6, EnpassantCapture, false, true, false, None)]
    #[case(E7, E8, KnightPromotion, false, false, false, Some(Knight))]
    #[case(E7, E8, BishopPromotion, false, false, false, Some(Bishop))]
    #[case(E7, E8, RookPromotion, false, false, false, Some(Rook))]
    #[case(E7, E8, QueenPromotion, false, false, false, Some(Queen))]
    #[case(E7, E8, KnightPromotionCapture, false, true, false, Some(Knight))]
    #[case(E7, E8, BishopPromotionCapture, false, true, false, Some(Bishop))]
    #[case(E7, E8, RookPromotionCapture, false, true, false, Some(Rook))]
    #[case(E7, E8, QueenPromotionCapture, false, true, false, Some(Queen))]
    fn pack_unpack(
        #[case] from: Square,
        #[case] to: Square,
        #[case] flag: MoveFlag,
        #[case] quiet: bool,
        #[case] capture: bool,
        #[case] reversible: bool,
        #[case] promotion_piece: Option<Piece>,
    ) {
        let position = Position::from_str(START_POSITION).unwrap();
        let mov = Move::new(from, to, flag);
        assert_eq!(mov.from(), from);
        assert_eq!(mov.to(), to);
        assert_eq!(mov.flag(), flag);
        assert_eq!(mov.is_quiet(), quiet);
        assert_eq!(mov.is_capture(), capture);
        assert_eq!(mov.is_reversible(&position), reversible);
        assert_eq!(mov.promotion_piece(), promotion_piece);
    }

    #[test]
    fn display() {
        let mov1 = Move::new(E4, E5, Quiet);
        assert_eq!(mov1.to_string(), "e4e5".to_string());

        let mov1 = Move::new(E7, D8, QueenPromotionCapture);
        assert_eq!(mov1.to_string(), "e7d8q".to_string());
    }

    #[rstest]
    #[case(C3, B5, Quiet, true)]
    #[case(C3, C3, Quiet, false)]
    #[case(C3, A6, Quiet, false)]
    #[case(D5, E6, Capture, true)]
    #[case(D5, C6, EnpassantCapture, true)]
    #[case(D4, E6, Capture, false)]
    #[case(E1, G1, KingsideCastle, true)]
    #[case(E1, C1, QueensideCastle, true)]
    #[case(C3, E4, Capture, false)]
    #[case(A6, E2, Capture, false)]
    #[case(E2, A6, Capture, true)]
    #[case(F6, E8, Capture, false)]
    #[case(A3, A4, Quiet, false)]
    #[case(A3, A4, Capture, false)]
    fn pseudo_legal(#[case] from: Square, #[case] to: Square, #[case] flag: MoveFlag, #[case] res: bool) {
        let position = Position::from_str(KIWIPETE_POSITION).unwrap();
        let mov = Move::new(from, to, flag);
        assert_eq!(mov.pseudo_legal(&position), res);
    }

    #[rstest]
    #[case("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2")]
    #[case("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3")]
    #[case("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2")]
    #[case("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2")]
    #[case("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2")]
    #[case("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9")]
    #[case("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4")]
    #[case("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")]
    #[case("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")]
    #[case("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1")]
    #[case("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1")]
    fn legal_are_pseudo_legal(#[case] fen: Fen) {
        let position = Position::try_from(fen.clone()).unwrap();
        position.moves(MoveStage::All).iter().for_each(|m| assert!(m.pseudo_legal(&position)));
    }
}
