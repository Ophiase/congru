use nalgebra::{Matrix3, Rotation3, UnitQuaternion};

use super::Match;
use crate::{Error, Point, RigidTransform};

const MINIMUM_PAIRS: usize = 3;
const RANK_TOLERANCE: f64 = 1e-12;

/// Weighted Kabsch–Umeyama. `svd` sorts singular values in decreasing
/// order, so the sign correction lands on the weakest axis.
pub(crate) fn kabsch(matches: &[Match]) -> Result<RigidTransform, Error> {
    if matches.len() < MINIMUM_PAIRS {
        return Err(Error::TooFewCorrespondences {
            found: matches.len(),
            needed: MINIMUM_PAIRS,
        });
    }
    let total: f64 = matches.iter().map(|pair| pair.weight).sum();
    if total.is_nan() || total <= 0.0 {
        return Err(Error::Degenerate);
    }
    let source_centre = weighted_mean(matches, total, |pair| pair.source);
    let target_centre = weighted_mean(matches, total, |pair| pair.target);
    let covariance = matches.iter().fold(Matrix3::zeros(), |sum, pair| {
        sum + (pair.source - source_centre)
            * (pair.target - target_centre).transpose()
            * pair.weight
    });
    let rotation = rotation_from_covariance(covariance)?;
    Ok(RigidTransform::from_parts(
        rotation,
        target_centre.coords - rotation * source_centre.coords,
    ))
}

fn rotation_from_covariance(covariance: Matrix3<f64>) -> Result<UnitQuaternion<f64>, Error> {
    let svd = covariance.svd(true, true);
    let (Some(left), Some(right_transposed)) = (svd.u, svd.v_t) else {
        return Err(Error::Degenerate);
    };
    if svd.singular_values.iter().any(|value| value.is_nan())
        || svd.singular_values[1] <= RANK_TOLERANCE * svd.singular_values[0]
    {
        return Err(Error::Degenerate);
    }
    let right = right_transposed.transpose();
    let handedness = (right * left.transpose()).determinant().signum();
    let correction = Matrix3::from_diagonal(&nalgebra::Vector3::new(1.0, 1.0, handedness));
    let rotation = Rotation3::from_matrix_unchecked(right * correction * left.transpose());
    Ok(UnitQuaternion::from_rotation_matrix(&rotation))
}

fn weighted_mean(matches: &[Match], total: f64, side: impl Fn(&Match) -> Point) -> Point {
    let sum = matches
        .iter()
        .fold(nalgebra::Vector3::zeros(), |sum, pair| {
            sum + side(pair).coords * pair.weight
        });
    Point::from(sum / total)
}
