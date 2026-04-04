use crate::evaluation::score::ValueScore;

pub mod nnue;
pub mod score;

pub static NNUE_PARAMS_BLOB: &str = include_str!("../../assets/dump/20260330-015436.nnue");
pub const MAX_POSITIONAL_WEIGHT: ValueScore = 320;

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::{
            MAX_POSITIONAL_WEIGHT, NNUE_PARAMS_BLOB, ValueScore,
            nnue::{NeuralNetwork, Parameters},
        },
        position::{MoveStage, Position, fen::START_POSITION},
    };
    use rstest::rstest;
    use std::{
        str::FromStr,
        sync::{Arc, LazyLock, Mutex},
    };

    static EVALUATOR: LazyLock<Arc<Mutex<NeuralNetwork>>> = LazyLock::new(|| {
        let params = Parameters::from_str(NNUE_PARAMS_BLOB).unwrap();
        Arc::new(Mutex::new(NeuralNetwork::new(params)))
    });

    #[rstest]
    #[case(START_POSITION, 20)]
    #[case("4rrk1/p1ppqpb1/B4np1/3nN3/8/8/PPPB1P1P/R3KQ1R w KQ -", -40)]
    #[case("1k6/8/1P6/3R3p/6p1/6r1/8/4K3 b - -", -80)]
    #[case("b7/2p2ppk/1p2p2p/p3P2P/P3pK2/bP2P3/2P1BPP1/3R4 w - -", 250)]
    #[case("8/1p6/2b5/2b5/2P5/8/2K2k2/8 b - -", -700)]
    #[case("8/p7/1p2k3/2p5/b1P1BP1B/3K4/1b6/8 w - -", 0)]
    #[case("r2q1rk1/p1p1bpp1/4p2p/2np4/5P2/4PR1P/PP2B1P1/RN2Q1K1 b - -", -100)]
    #[case("rnbqkbnr/ppp2p1p/6p1/3pPp2/3P4/5N2/PPP1B1PP/RN1QK2R w KQkq -", -400)]
    #[case("rnb2rk1/2pnbppp/1p2p3/pP6/P3P3/2NB1N2/5PPP/R1BQK2R b KQ -", 1000)]
    #[case("2bq1rk1/4pp1p/6p1/p2p4/Pp6/1P2PNP1/1Q3PBP/R5K1 w - -", 300)]
    #[case("3r4/1pk3p1/p4pB1/4p1p1/bPPr4/3P3P/5PP1/2R3K1 w - -", -550)]
    #[case("1rbqk1nr/p1pp1p1p/2p3p1/8/P2bP3/6P1/2P2PBP/RNB1K2R w KQk -", -1100)]
    #[case("2r3k1/pq3ppp/8/1pp4Q/8/3P2P1/Pb2PP1P/1R1R2K1 b - -", 250)]
    #[case("rnbb1rk1/pp4pp/4pn2/2p2p2/2P4N/2N3P1/PP2PPBP/R1B2RK1 b - -", 50)]
    #[case("4r3/6b1/1p3kpp/pN1R4/P6P/2P3P1/1P3K2/8 w - -", 150)]
    #[case("2b2r2/6kp/3q2p1/1p1p1r2/p1pPp3/P3P2P/1P3PP1/QR3RK1 w - -", -550)]
    #[case("r1bq1rk1/pp1nppbp/5np1/1B1p4/3P1B2/4PN1P/PPPQ1PP1/R4RK1 b - -", -250)]
    #[case("rn2qrk1/pb2bppp/1pp1p3/8/2BP4/4PNN1/PP3PPP/2RQ2K1 b - -", -500)]
    #[case("3r1rk1/1p2bpp1/7p/1N1p3P/P2R4/6P1/1P2PP2/3R2K1 b - -", 200)]
    #[case("8/4pp2/4k1p1/1R5p/4KP1P/4P1P1/8/r7 b - -", 0)]
    fn eval_in_range(#[case] position: &str, #[case] expected: ValueScore) {
        let position = Position::from_str(position).unwrap();

        assert_eq!(position.moves(MoveStage::CapturesAndPromotions).len(), 0, "position is not quiet");

        // Coincidentally, this also tests that the evaluate method
        // correctly updates the NNUE parameters in-between unrelated positions.
        let evaluation = EVALUATOR.lock().unwrap().evaluate(&position);
        assert!(
            (evaluation - expected).abs() <= MAX_POSITIONAL_WEIGHT,
            "got evaluation {} for position {}, expected around {}",
            evaluation,
            position.fen(),
            expected
        );
    }
}
