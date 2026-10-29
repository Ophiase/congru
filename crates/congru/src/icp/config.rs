use crate::{Distance, Fraction, Kernel, Prior};

/// What a correspondence's residual measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Metric {
    /// Distance to the matched point (Besl–McKay). Needs no normals;
    /// converges slowly on smooth surfaces.
    PointToPoint,
    /// Distance to the tangent plane of the matched point (Chen–Medioni).
    /// Needs target normals; converges in a few iterations on surfaces.
    #[default]
    PointToPlane,
}

/// When the iteration stops: the last step moved less than both tolerances,
/// or the iteration budget ran out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Convergence {
    pub max_iterations: u32,
    /// Smallest rotation step, in radians, still worth another iteration.
    pub rotation: f64,
    /// Smallest translation step, in coordinate units, still worth another
    /// iteration.
    pub translation: f64,
}

impl Default for Convergence {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            rotation: 1e-7,
            translation: 1e-7,
        }
    }
}

/// Iterative closest point: pair each source point with its nearest target
/// point, reject and weigh the pairs, solve for the motion, repeat.
///
/// The defaults suit a clean scan with a good initial pose; on real data set
/// at least a robust [`Kernel`] or a correspondence distance.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Icp {
    pub metric: Metric,
    pub kernel: Kernel,
    /// Pairs farther apart than this are dropped; `None` keeps them all.
    pub max_correspondence_distance: Option<Distance>,
    /// Share of the source expected to overlap the target: only that share
    /// of closest pairs is kept (trimmed ICP). [`Fraction::ALL`] trims
    /// nothing.
    pub overlap: Fraction,
    pub convergence: Convergence,
    /// Point-to-plane only: keeps the pose near the initial one, as a
    /// tracker keeps it near its prediction.
    pub prior: Option<Prior>,
}
