use crate::{evaluation::ValueScore, search::pvs::NodeType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Window {
    alpha: ValueScore,
    beta: ValueScore,
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
        }
    }
}

impl Window {
    pub fn best(&self) -> ValueScore {
        self.alpha
    }

    pub fn reverse(&self) -> Self {
        Window {
            alpha: -self.beta,
            beta: -self.alpha,
        }
    }

    pub fn reverse_null(&self) -> Self {
        Window {
            alpha: -self.alpha - 1,
            beta: -self.alpha,
        }
    }

    pub fn is_null(&self) -> bool {
        self.alpha == self.beta - 1
    }

    pub fn requires_full_search(&self, null_score: ValueScore) -> bool {
        null_score > self.alpha && null_score < self.beta
    }

    pub fn feed(&mut self, score: ValueScore) -> FeedResult {
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
    use super::*;

    #[test]
    fn round_trip() {
        let mut window = Window::default();
        assert_eq!(window, window.reverse());
        assert_eq!(window, window.reverse().reverse());

        window.feed(300);
        assert_ne!(window, window.reverse());
        assert_eq!(window, window.reverse().reverse());
    }

    #[test]
    fn feed() {
        let mut window = Window::default();

        // We, say white, see a position where we are +300.
        assert_eq!(window.feed(300), FeedResult::Improvement);
        assert_eq!(window.best(), 300);

        // Another position is not better than the first one.
        assert_eq!(window.feed(200), FeedResult::FailLow);
        assert_eq!(window.best(), 300);

        // Let's backtrack and analyze another move. Black to play.
        window = window.reverse();

        // We discovered a position where black is +200.
        // We can prune this branch, as it's for sure not the best.
        assert_eq!(window.feed(200), FeedResult::FailHigh);
        assert_eq!(window.best(), 200);
    }

    #[test]
    fn feed_cache() {
        let mut window = Window::default();

        // An exact score is immediately returned.
        assert_eq!(window.feed_cache(300, NodeType::PVNode), Some(300));
        assert_eq!(window.best(), ValueScore::MIN + 1);

        // We failed high at this node, so we know that this is a lowerbound of the score.
        assert_eq!(window.feed_cache(300, NodeType::CutNode), None);
        assert_eq!(window.best(), 300);
        assert_eq!(window.feed_cache(200, NodeType::CutNode), None);
        assert_eq!(window.best(), 300);

        // We failed low at this node, so we know that this is an upperbound of the score.
        assert_eq!(window.feed_cache(500, NodeType::AllNode), None);
        assert_eq!(window.best(), 300);
        assert_eq!(window.beta, 500);

        // We now fail high outside the window.
        let mut window2 = window.clone();
        assert_eq!(window2.feed_cache(400, NodeType::CutNode), None);
        assert_eq!(window2.feed_cache(600, NodeType::CutNode), Some(600));

        // We now fail low outside the window.
        let mut window3 = window.clone();
        assert_eq!(window3.feed_cache(400, NodeType::AllNode), None);
        assert_eq!(window3.feed_cache(200, NodeType::AllNode), Some(200));
    }

    #[test]
    fn zero_search() {
        let mut window = Window::default();
        assert!(!window.is_null());

        // Find a position where we, say white, are +300.
        window.feed(300);

        // Try another move, assuming it won't be the best ("principal variation").
        // Any move that refutes this assumption suffices.
        let null_window = window.reverse_null();
        assert_eq!(
            null_window,
            Window {
                alpha: -301,
                beta: -300
            }
        );
        assert!(null_window.is_null());

        // We'll require a research with a non-null window if our assumption was incorrect.
        assert!(window.requires_full_search(350));
        assert!(!window.requires_full_search(250));
    }
}
