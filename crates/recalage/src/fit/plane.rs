use nalgebra::{Matrix6, UnitQuaternion, Vector6};

use super::Match;
use crate::geometry::{count_as_float, mean};
use crate::{Error, Point, RigidTransform};

const MINIMUM_PAIRS: usize = 6;
const CONDITION_FLOOR: f64 = 1e-12;

/// One Gauss–Newton step of point-to-plane: the small motion minimising
/// `Σ wᵢ (nᵢ · (R sᵢ + t − tᵢ))²` with `R ≈ I + [ω]×`, solved about the
/// source centroid so the normal equations stay well conditioned.
pub(crate) fn point_to_plane(matches: &[Match]) -> Result<RigidTransform, Error> {
    if matches.len() < MINIMUM_PAIRS {
        return Err(Error::TooFewCorrespondences {
            found: matches.len(),
            needed: MINIMUM_PAIRS,
        });
    }
    let sources: Vec<Point> = matches.iter().map(|pair| pair.source).collect();
    let centre = mean(&sources);
    let (hessian, gradient) = normal_equations(matches, &centre)?;
    let eigenvalues = hessian.symmetric_eigenvalues();
    if eigenvalues.iter().any(|value| value.is_nan())
        || eigenvalues.min() <= CONDITION_FLOOR * eigenvalues.max().max(f64::MIN_POSITIVE)
    {
        return Err(Error::Degenerate);
    }
    let step = hessian
        .cholesky()
        .ok_or(Error::Degenerate)?
        .solve(&-gradient);
    let rotation = UnitQuaternion::from_scaled_axis(step.fixed_rows::<3>(0).into_owned());
    let translation = step.fixed_rows::<3>(3).into_owned();
    Ok(RigidTransform::from_parts(
        rotation,
        translation + centre.coords - rotation * centre.coords,
    ))
}

fn normal_equations(
    matches: &[Match],
    centre: &Point,
) -> Result<(Matrix6<f64>, Vector6<f64>), Error> {
    let mut hessian = Matrix6::zeros();
    let mut gradient = Vector6::zeros();
    for pair in matches {
        let normal = pair.normal.ok_or(Error::MissingNormals)?;
        let lever = pair.source - centre;
        let jacobian =
            Vector6::from_iterator(lever.cross(&normal).iter().chain(normal.iter()).copied());
        let residual = normal.dot(&(pair.source - pair.target));
        hessian += jacobian * jacobian.transpose() * pair.weight;
        gradient += jacobian * (residual * pair.weight);
    }
    let scale = count_as_float(matches.len());
    Ok((hessian / scale, gradient / scale))
}
