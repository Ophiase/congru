use crate::{Kernel, Pose, Real};

/// Below this many spacings of the scalar, a step is rounding and not motion.
const ROUNDING_STEPS: f64 = 16.0;
/// The tolerance `f64` is given, the same as the main crate's.
const FINEST: f64 = 1e-7;

/// What a correspondence's residual measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Metric {
    /// Distance to the matched point. Needs no normals; slow on smooth
    /// surfaces.
    PointToPoint,
    /// Distance to the tangent plane of the matched point. Needs target
    /// normals; a few iterations on surfaces.
    #[default]
    PointToPlane,
}

/// When the iteration stops: the last step moved less than both tolerances,
/// or the budget ran out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Convergence<Float> {
    pub max_iterations: u32,
    /// Smallest rotation step, in radians, still worth another iteration.
    pub rotation: Float,
    /// Smallest translation step, in coordinate units.
    pub translation: Float,
}

impl<Float: Real> Convergence<Float> {
    /// The tolerances of a scene of half-diagonal `extent`: never finer than
    /// the scalar resolves, or `f32` would spend its budget on rounding.
    #[must_use]
    pub fn for_extent(extent: Float) -> Self {
        let rotation = Float::of(FINEST).larger(Float::of(ROUNDING_STEPS) * Float::EPSILON);
        Self {
            max_iterations: 50,
            rotation,
            translation: rotation * extent,
        }
    }
}

impl<Float: Real> Default for Convergence<Float> {
    fn default() -> Self {
        Self::for_extent(Float::ONE)
    }
}

/// Iterative closest point: pair each source point with its nearest target
/// point, drop and weigh the pairs, solve for the motion, repeat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Icp<Float> {
    pub metric: Metric,
    pub kernel: Kernel<Float>,
    /// Pairs at least this far apart are dropped; `None` keeps them all.
    pub gate: Option<Float>,
    pub convergence: Convergence<Float>,
}

impl<Float: Real> Default for Icp<Float> {
    fn default() -> Self {
        Self {
            metric: Metric::default(),
            kernel: Kernel::default(),
            gate: None,
            convergence: Convergence::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Converged,
    IterationLimit,
}

/// A pose found, and what is known about it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment<Float> {
    pub pose: Pose<Float>,
    /// Root mean square distance of the pairs within the gate.
    pub rmse: Float,
    /// Share of the source with a pair within the gate.
    pub inlier_fraction: Float,
    pub iterations: u32,
    pub stop: StopReason,
}
