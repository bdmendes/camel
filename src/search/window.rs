use crate::{evaluation::score::ValueScore, moves::Move, search::NodeType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Window {
    alpha: ValueScore,
    beta: ValueScore,
    best_score: ValueScore,
    best_move: Option<Move>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FeedResult {
    Improvement,
    FailHigh,
    FailLow,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            alpha: ValueScore::MIN + 1,
            beta: ValueScore::MAX,
            best_score: ValueScore::MIN + 1,
            best_move: None,
        }
    }
}

impl Window {
    pub fn new(alpha: ValueScore, beta: ValueScore) -> Self {
        Self {
            alpha: alpha.max(ValueScore::MIN + 1),
            beta: beta.max(ValueScore::MIN + 1),
            best_score: ValueScore::MIN + 1,
            best_move: None,
        }
    }

    pub fn alpha(&self) -> ValueScore {
        self.alpha
    }

    pub fn best(&self) -> ValueScore {
        self.best_score
    }

    pub fn best_move(&self) -> Option<Move> {
        self.best_move
    }

    pub fn reverse(&self) -> Self {
        Window {
            alpha: -self.beta,
            beta: -self.alpha,
            best_score: ValueScore::MIN + 1,
            best_move: None,
        }
    }

    pub fn reverse_null_around_beta(&self) -> Self {
        Window {
            alpha: -self.beta,
            beta: -self.beta + 1,
            best_score: ValueScore::MIN + 1,
            best_move: None,
        }
    }

    pub fn reverse_null_around_alpha(&self) -> Self {
        Window {
            alpha: -self.alpha - 1,
            beta: -self.alpha,
            best_score: ValueScore::MIN + 1,
            best_move: None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.alpha == self.beta - 1
    }

    pub fn cuts_off(&self, score: ValueScore) -> bool {
        score >= self.beta
    }

    pub fn improves(&self, score: ValueScore) -> bool {
        score > self.alpha
    }

    pub fn feed(&mut self, score: ValueScore, mov: Option<Move>) -> FeedResult {
        if score > self.best_score {
            self.best_score = score;
            self.best_move = mov;
        }

        if score > self.alpha {
            self.alpha = score;
            if self.alpha >= self.beta {
                FeedResult::FailHigh
            } else {
                FeedResult::Improvement
            }
        } else {
            FeedResult::FailLow
        }
    }

    pub fn feed_cache(&mut self, score: ValueScore, node_type: NodeType) -> Option<ValueScore> {
        match node_type {
            NodeType::PVNode => Some(score),
            NodeType::CutNode => {
                self.alpha = self.alpha.max(score);
                (self.alpha >= self.beta).then_some(self.alpha)
            }
            NodeType::AllNode => {
                self.beta = self.beta.min(score);
                (self.alpha >= self.beta).then_some(self.beta)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{moves::MoveFlag, position::square::Square};

    use super::*;

    #[test]
    fn new() {
        let window = Window::new(-100, 100);
        assert_eq!(window.alpha, -100);
        assert_eq!(window.beta, 100);
        assert_eq!(window.best(), ValueScore::MIN + 1);
    }

    #[test]
    fn new_edges() {
        let window = Window::new(ValueScore::MIN, ValueScore::MAX);
        assert_eq!(window.alpha, ValueScore::MIN + 1);
        assert_eq!(window.beta, ValueScore::MAX);
        assert_eq!(window.best(), ValueScore::MIN + 1);
    }

    #[test]
    fn feed_reverse() {
        let mut window = Window::default();

        // We, say white, see a position where we are +300.
        assert_eq!(window.feed(300, None), FeedResult::Improvement);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.best(), 300);
        assert!(window.improves(400));
        assert!(!window.improves(300));

        // A position with the same score is not considered an improvement.
        assert_eq!(window.feed(300, None), FeedResult::FailLow);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.best(), 300);

        // Another position is not better than the first one.
        assert_eq!(window.feed(200, None), FeedResult::FailLow);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.best(), 300);

        // Let's backtrack and analyze another move. Black to play.
        window = window.reverse();
        assert_eq!(window.alpha, ValueScore::MIN + 1);
        assert_eq!(window.beta, -300);
        assert_eq!(window.best(), ValueScore::MIN + 1);
        assert!(window.cuts_off(-200));
        assert!(!window.cuts_off(-400));

        // We discovered a position where black is +200.
        // We can prune this branch, as it's for sure not the best.
        assert_eq!(window.feed(200, None), FeedResult::FailHigh);
        assert_eq!(window.alpha, 200);
        assert_eq!(window.best(), 200);
    }

    #[test]
    fn feed_moves() {
        let mut window = Window::default();
        let mov = Move::new(Square::E2, Square::E4, MoveFlag::Quiet);

        window.feed(300, None);
        assert_eq!(window.best_move(), None);

        window.feed(400, Some(mov));
        assert_eq!(window.best_move(), Some(mov));

        let mov2 = Move::new(Square::D2, Square::D4, MoveFlag::Quiet);
        window.feed(350, Some(mov2));
        assert_eq!(window.best_move(), Some(mov));
    }

    #[test]
    fn feed_cache() {
        let mut window = Window::default();

        // An exact score is immediately returned.
        assert_eq!(window.feed_cache(300, NodeType::PVNode), Some(300));
        assert_eq!(window.alpha, ValueScore::MIN + 1);
        assert_eq!(window.best(), ValueScore::MIN + 1);

        // We failed high at this node, so we know that this is a lowerbound of the score.
        assert_eq!(window.feed_cache(300, NodeType::CutNode), None);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.best(), ValueScore::MIN + 1);
        assert_eq!(window.feed_cache(200, NodeType::CutNode), None);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.best(), ValueScore::MIN + 1);

        // We failed low at this node, so we know that this is an upperbound of the score.
        assert_eq!(window.feed_cache(500, NodeType::AllNode), None);
        assert_eq!(window.alpha, 300);
        assert_eq!(window.beta, 500);
        assert_eq!(window.best(), ValueScore::MIN + 1);

        // We now fail high outside the window.
        let mut window2 = window;
        assert_eq!(window2.feed_cache(400, NodeType::CutNode), None);
        assert_eq!(window2.feed_cache(600, NodeType::CutNode), Some(600));

        // We now fail low outside the window.
        let mut window3 = window;
        assert_eq!(window3.feed_cache(400, NodeType::AllNode), None);
        assert_eq!(window3.feed_cache(200, NodeType::AllNode), Some(200));
    }

    #[test]
    fn zero_search_beta_surpass() {
        let window = Window::default();
        assert!(!window.is_null());

        let null_window = window.reverse_null_around_beta();
        assert_eq!(
            null_window,
            Window {
                alpha: -ValueScore::MAX,
                beta: -ValueScore::MAX + 1,
                best_score: ValueScore::MIN + 1,
                best_move: None,
            }
        );
        assert!(null_window.is_null());
    }

    #[test]
    fn zero_search_alpha_surpass() {
        let window = Window::default();
        assert!(!window.is_null());

        let null_window = window.reverse_null_around_alpha();
        assert_eq!(
            null_window,
            Window {
                alpha: -(ValueScore::MIN + 1) - 1,
                beta: -(ValueScore::MIN + 1),
                best_score: ValueScore::MIN + 1,
                best_move: None,
            }
        );
        assert!(null_window.is_null());
    }
}
