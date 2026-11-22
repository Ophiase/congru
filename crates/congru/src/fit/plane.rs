use nalgebra::{Matrix6, UnitQuaternion, Vector6};

use super::{Match, Prior};
use crate::geometry::{count_as_float, mean};
use crate::{Error, Point, RigidTransform};

const MINIMUM_PAIRS: usize = 6;
const DAMPING: f64 = 1e-4;

/// One Gauss–Newton step of point-to-plane: the small motion minimising
/// `Σ wᵢ (nᵢ · (R sᵢ + t − tᵢ))²` with `R ≈ I + [ω]×`, solved about the
/// source centroid so the normal equations stay well conditioned. The step
/// is damped: along a motion the pairs leave free (a slide over a plane),
/// it stays near zero, keeping the initial pose there instead of failing.
/// With an anchor, the step is also pulled back toward the initial pose,
/// `deviation` away from the current one.
pub(crate) fn point_to_plane(
    matches: &[Match],
    anchor: Option<(&Prior, &RigidTransform)>,
) -> Result<RigidTransform, Error> {
    if matches.len() < MINIMUM_PAIRS {
        return Err(Error::TooFewCorrespondences {
            found: matches.len(),
            needed: MINIMUM_PAIRS,
        });
    }
    let sources: Vec<Point> = matches.iter().map(|pair| pair.source).collect();
    let centre = mean(&sources);
    let (mut hessian, mut gradient) = normal_equations(matches, &centre)?;
    if let Some((prior, deviation)) = anchor {
        let (stiffness, pull) = prior.pull(matches.len(), deviation, &centre);
        hessian += stiffness;
        gradient += pull;
    }
    let stiffest = hessian.symmetric_eigenvalues().max();
    if stiffest.is_nan() || stiffest <= 0.0 {
        return Err(Error::Degenerate);
    }
    let damped = hessian + Matrix6::identity() * (DAMPING * stiffest);
    let step = damped
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

pub(super) fn normal_equations(
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
