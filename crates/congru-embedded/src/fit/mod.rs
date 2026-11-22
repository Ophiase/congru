mod closed_form;
mod kernel;
mod moments;
mod plane;

pub use closed_form::{Similarity, kabsch, similarity};
pub use kernel::Kernel;
pub(crate) use plane::point_to_plane;

use crate::Vector3;

/// One correspondence: `source` should land on `target`. A weight must be
/// finite and not negative; zero means the pair does not count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair<Float> {
    pub source: Vector3<Float>,
    pub target: Vector3<Float>,
    pub weight: Float,
}
