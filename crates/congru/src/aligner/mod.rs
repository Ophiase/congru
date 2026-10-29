mod adaptive;
mod annealed;
mod guarded;
mod local;
mod search;

use crate::{Alignment, Error, Observability, PointCloud, RigidTransform};

pub use adaptive::Adaptive;
pub use annealed::Annealed;
pub use guarded::Guarded;
pub use local::Local;
pub use search::Search;

/// One frame handed to an [`Aligner`].
#[derive(Debug, Clone, Copy)]
pub struct Frame<'scan> {
    /// What the sensor returned, in the sensor's frame.
    pub scan: &'scan PointCloud,
    /// A pose of the model to start from, when the caller has one: a
    /// detector, an encoder, the robot's kinematics.
    pub guess: Option<RigidTransform>,
}

/// An aligner's answer for one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    /// Maps the model's frame to the sensor's.
    pub pose: RigidTransform,
    /// Whether the method stands behind the pose; a method that knows it
    /// failed says so here rather than returning an error.
    pub trusted: bool,
    pub observability: Option<Observability>,
}

/// An alignment method. It owns its model, is handed frames one at a time,
/// and may remember past frames, as a tracker keeps its filter; `reset`
/// forgets them.
pub trait Aligner {
    /// Short and stable: it names the method's row in every report.
    fn name(&self) -> &str;

    /// # Errors
    /// When the method produces no pose at all for this frame.
    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error>;

    fn reset(&mut self) {}
}

/// Share of the scan an alignment must keep for a method to trust it.
const TRUSTED_SHARE: f64 = 0.5;

impl Estimate {
    /// From an alignment of the scan onto the model, trusted when it keeps
    /// at least half the scan.
    #[must_use]
    pub fn from_alignment(alignment: Alignment) -> Self {
        Self {
            pose: alignment.transform.inverse(),
            trusted: alignment.inlier_fraction >= TRUSTED_SHARE,
            observability: alignment.observability,
        }
    }
}

fn start(frame: &Frame<'_>) -> RigidTransform {
    frame
        .guess
        .map_or_else(RigidTransform::identity, |guess| guess.inverse())
}
