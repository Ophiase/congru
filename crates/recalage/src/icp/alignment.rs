use crate::RigidTransform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Converged,
    IterationLimit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    /// The motion taking the source onto the target.
    pub transform: RigidTransform,
    /// Root mean square distance between the kept pairs, at the final pose.
    pub rmse: f64,
    /// Share of source points with a kept pair at the final pose: a low value
    /// flags a wrong local minimum or a small overlap.
    pub inlier_fraction: f64,
    pub iterations: u32,
    pub stop: StopReason,
}
