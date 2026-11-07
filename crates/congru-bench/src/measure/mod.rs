pub mod breaking;
pub mod certified;
mod realistic;
mod registration;
mod stats;
mod tracking;

pub use realistic::spot;
pub use registration::{register, scene};
pub use stats::Summary;
pub use tracking::track;

/// A pose is a success within both tolerances of the truth.
pub const ROTATION_TOLERANCE_DEGREES: f64 = 2.0;
/// As a share of the model's size.
pub const TRANSLATION_TOLERANCE: f64 = 0.02;

/// One alignment, scored against the truth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trial {
    /// Infinite when the method gave no pose.
    pub rotation_degrees: f64,
    /// As a share of the model's size; infinite when no pose.
    pub translation: f64,
    pub trusted: bool,
    pub seconds: f64,
}

impl Trial {
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.rotation_degrees < ROTATION_TOLERANCE_DEGREES
            && self.translation < TRANSLATION_TOLERANCE
    }

    /// The worst outcome for a system that acts on the pose: wrong, and
    /// claimed right.
    #[must_use]
    pub fn confidently_wrong(&self) -> bool {
        self.trusted && !self.succeeded()
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "a rounded, non-negative count of points, far below 2^52"
)]
#[must_use]
pub fn share_of(count: usize, share: f64) -> usize {
    (count as f64 * share).round().max(0.0) as usize
}
