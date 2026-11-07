mod kabsch;
mod kernel;
mod observability;
mod plane;
mod prior;
mod similarity;

use nalgebra::Unit;

use crate::geometry::count_as_float;
use crate::{Error, Point, RigidTransform, Vector};

pub(crate) use kabsch::kabsch;
pub use kernel::Kernel;
pub(crate) use observability::observe;
pub use observability::{MotionKind, Observability, WeakMotion};
pub(crate) use plane::point_to_plane;
pub use prior::Prior;
pub use similarity::{SimilarFit, fit_similarity};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Match {
    pub source: Point,
    pub target: Point,
    pub normal: Option<Unit<Vector>>,
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub transform: RigidTransform,
    /// Weighted root mean square distance left after the motion; for
    /// fiducials, the fiducial registration error.
    pub rmse: f64,
}

/// The rigid motion minimising `Σ wᵢ ‖R sᵢ + t − tᵢ‖²` (Kabsch–Umeyama),
/// each `sourceᵢ` paired with `targetᵢ`, uniform weights when `None`.
///
/// # Errors
/// [`Error::LengthMismatch`], [`Error::TooFewCorrespondences`] below three
/// pairs, [`Error::InvalidParameter`] for a negative or non-finite weight,
/// [`Error::Degenerate`] for collinear points.
pub fn fit_rigid(
    source: &[Point],
    target: &[Point],
    weights: Option<&[f64]>,
) -> Result<Fit, Error> {
    check_length("source and target", source.len(), target.len())?;
    let weights = match weights {
        Some(weights) => {
            check_length("points and weights", source.len(), weights.len())?;
            check_weights(weights)?;
            weights.to_vec()
        }
        None => vec![1.0; source.len()],
    };
    let matches: Vec<Match> = source
        .iter()
        .zip(target)
        .zip(weights)
        .map(|((source, target), weight)| Match {
            source: *source,
            target: *target,
            normal: None,
            weight,
        })
        .collect();
    let transform = kabsch(&matches)?;
    Ok(Fit {
        transform,
        rmse: weighted_rmse(&matches, &transform),
    })
}

fn check_length(what: &'static str, left: usize, right: usize) -> Result<(), Error> {
    if left == right {
        Ok(())
    } else {
        Err(Error::LengthMismatch { what, left, right })
    }
}

fn check_weights(weights: &[f64]) -> Result<(), Error> {
    match weights
        .iter()
        .find(|weight| !(weight.is_finite() && **weight >= 0.0))
    {
        Some(weight) => Err(Error::InvalidParameter {
            name: "weight",
            expected: "finite and non-negative",
            value: *weight,
        }),
        None => Ok(()),
    }
}

fn weighted_rmse(matches: &[Match], transform: &RigidTransform) -> f64 {
    let (squared, total) = matches.iter().fold((0.0, 0.0), |(squared, total), pair| {
        let gap = (transform.apply(&pair.source) - pair.target).norm_squared();
        (squared + pair.weight * gap, total + pair.weight)
    });
    if total > 0.0 {
        (squared / total).sqrt()
    } else {
        (squared / count_as_float(matches.len().max(1))).sqrt()
    }
}
