use congru::{Alignment, RigidTransform};

/// Which of the two phases produced a frame's pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Predicted from the last frames, refined by local iterations pulled
    /// toward the prediction.
    Follow,
    /// Searched over all rotations: the first frame, or a frame the follow
    /// phase could not explain.
    Find,
}

/// One frame's outcome. `pose` maps the model's frame to the sensor's.
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub pose: RigidTransform,
    pub phase: Phase,
    /// Whether the next frame may follow from this one.
    pub trusted: bool,
    pub alignment: Alignment,
}

impl Update {
    /// What the frame alone says of the pose, before the tracker's filter: `pose`
    /// is steadier and lags behind what a jolt of the camera did at once.
    #[must_use]
    pub fn measured(&self) -> RigidTransform {
        self.alignment.transform.inverse()
    }

    /// Whether the pose can be stood by: followed from, and with no motion the data
    /// left free. A smooth blob or a flat patch can be followed for ever and turn
    /// freely under the scan: that is an answer to refuse, not an error to count.
    #[must_use]
    pub fn defensible(&self) -> bool {
        self.trusted
            && self
                .alignment
                .observability
                .as_ref()
                .is_some_and(|report| report.weak_motions.is_empty())
    }
}
