use crate::evaluation::ValueScore;

#[derive(Debug, PartialEq, Eq)]
pub struct Window {
    alpha: ValueScore,
    beta: ValueScore,
}

impl Window {
    pub fn new() -> Self {
        Window {
            alpha: ValueScore::MIN + 1,
            beta: ValueScore::MAX,
        }
    }

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

    pub fn feed_cutoff(&mut self, score: ValueScore) -> Option<ValueScore> {
        self.alpha = self.alpha.max(score);
        (self.alpha >= self.beta).then(|| self.alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let window = Window::new();
        assert_eq!(window, window.reverse().reverse());
    }

    #[test]
    fn cutoff() {
        let mut window = Window::new();

        // We, say white, see a position where we are +300.
        assert_eq!(window.feed_cutoff(300), None);
        assert_eq!(window.best(), 300);

        // Let's backtrack and analyze another move. Black to play.
        window = window.reverse();

        // We discovered a position where black is +200.
        // We can prune this branch, as it's for sure not the best.
        assert_eq!(window.feed_cutoff(200), Some(200));
    }

    #[test]
    fn zero_search() {
        let mut window = Window::new();
        assert!(!window.is_null());

        // Find a position where we, say white, are +300.
        window.feed_cutoff(300);

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
