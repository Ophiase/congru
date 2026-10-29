mod registration;
mod tracking;

pub use registration::{SITUATIONS, Situation};
pub use tracking::{Sequence, sequences};

/// Seeds the methods may be tuned on, and seeds the published tables use:
/// a method tuned on the published seeds would be graded on its homework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seeds {
    Tuning,
    Evaluation,
}

impl Seeds {
    #[must_use]
    pub fn range(self, count: u64) -> std::ops::Range<u64> {
        match self {
            Self::Tuning => 0..count,
            Self::Evaluation => 1_000_000..1_000_000 + count,
        }
    }
}
