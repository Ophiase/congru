use super::Pair;
use super::moments::moments;
use crate::linear::symmetric_eigen;
use crate::{Error, Pose, Real, UnitQuaternion, Vector3};

/// A motion and a scale: `point ↦ scale · rotation · point + translation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Similarity<Float> {
    pub scale: Float,
    pub pose: Pose<Float>,
}

impl<Float: Real> Similarity<Float> {
    #[must_use]
    pub fn apply(&self, point: Vector3<Float>) -> Vector3<Float> {
        let turned = self
            .pose
            .rotation
            .matrix()
            .map(|row| Vector3(row).dot(point));
        Vector3(turned) * self.scale + self.pose.translation
    }
}

/// The motion minimising `Σ wᵢ ‖R sᵢ + t − tᵢ‖²`, in closed form. The
/// iterator is walked twice, so it can compute pairs on the fly and needs no
/// buffer.
///
/// # Errors
/// [`Error::TooFewPairs`] below three pairs of positive weight,
/// [`Error::NotFinite`] for a negative or non-finite weight,
/// [`Error::Degenerate`] for collinear points, or points so nearly collinear
/// that `Float` cannot resolve the rotation about their line.
pub fn kabsch<Float: Real>(
    pairs: impl Iterator<Item = Pair<Float>> + Clone,
) -> Result<Pose<Float>, Error> {
    let moments = moments(pairs)?;
    let (rotation, _) = best_rotation(&moments.covariance)?;
    Ok(about_centres(
        rotation,
        Float::ONE,
        moments.source_centre,
        moments.target_centre,
    ))
}

/// The motion and scale minimising `Σ wᵢ ‖c R sᵢ + t − tᵢ‖²` ([Umeyama]).
///
/// [Umeyama]: https://doi.org/10.1109/34.88573
///
/// # Errors
/// The same as [`kabsch`].
pub fn similarity<Float: Real>(
    pairs: impl Iterator<Item = Pair<Float>> + Clone,
) -> Result<Similarity<Float>, Error> {
    let moments = moments(pairs)?;
    let (rotation, aligned) = best_rotation(&moments.covariance)?;
    if !moments.source_spread.strictly_positive() {
        return Err(Error::Degenerate);
    }
    let scale = aligned / moments.source_spread;
    Ok(Similarity {
        scale,
        pose: about_centres(
            rotation,
            scale,
            moments.source_centre,
            moments.target_centre,
        ),
    })
}

fn about_centres<Float: Real>(
    rotation: UnitQuaternion<Float>,
    scale: Float,
    source_centre: Vector3<Float>,
    target_centre: Vector3<Float>,
) -> Pose<Float> {
    let turned = rotation.matrix().map(|row| Vector3(row).dot(source_centre));
    Pose {
        rotation,
        translation: target_centre - Vector3(turned) * scale,
    }
}

/// [Horn]'s quaternion: the top eigenvector of a symmetric 4 × 4 made from
/// the covariance. It can only be a proper rotation, so no reflection case
/// remains, and its eigenvalue is `Σ wᵢ (R s̃ᵢ) · t̃ᵢ`, what the scale needs.
/// The gap to the next eigenvalue is `2 (σ₂ + σ₃)` of the covariance: below
/// `√ε` of the top, rounding alone turns the answer about the data's line.
///
/// [Horn]: https://doi.org/10.1364/JOSAA.4.000629
fn best_rotation<Float: Real>(
    covariance: &[[Float; 3]; 3],
) -> Result<(UnitQuaternion<Float>, Float), Error> {
    let eigen = symmetric_eigen(horn_matrix(covariance));
    let (mut top, mut top_value) = (0, -Float::INFINITY);
    let mut next_value = -Float::INFINITY;
    for (index, value) in eigen.values.into_iter().enumerate() {
        if value > top_value {
            (next_value, top_value, top) = (top_value, value, index);
        } else if value > next_value {
            next_value = value;
        }
    }
    let gap = top_value - next_value;
    if !top_value.strictly_positive() || gap <= Float::EPSILON.root() * top_value {
        return Err(Error::Degenerate);
    }
    Ok((UnitQuaternion::normalised(eigen.column(top)), top_value))
}

#[expect(clippy::similar_names, reason = "named after the matrix's indices")]
fn horn_matrix<Float: Real>(covariance: &[[Float; 3]; 3]) -> [[Float; 4]; 4] {
    let [[xx, xy, xz], [yx, yy, yz], [zx, zy, zz]] = *covariance;
    let (twist_x, twist_y, twist_z) = (yz - zy, zx - xz, xy - yx);
    let (sum_xy, sum_xz, sum_yz) = (xy + yx, xz + zx, yz + zy);
    [
        [xx + yy + zz, twist_x, twist_y, twist_z],
        [twist_x, xx - yy - zz, sum_xy, sum_xz],
        [twist_y, sum_xy, yy - xx - zz, sum_yz],
        [twist_z, sum_xz, sum_yz, zz - xx - yy],
    ]
}
