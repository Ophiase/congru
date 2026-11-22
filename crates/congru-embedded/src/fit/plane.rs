use super::Pair;
use crate::linear::solve_positive;
use crate::{Error, Pose, Real, UnitQuaternion, Vector3};

const MINIMUM_PAIRS: usize = 6;
const DAMPING: f64 = 1e-4;

/// One Gauss–Newton step of point-to-plane about the sources' centre, levers
/// divided by their radius so rotation and translation weigh alike in the
/// normal equations whatever the unit, which `f32` needs. Damped: along a
/// motion the pairs leave free, the step stays near zero. The fixed point is
/// where the gradient vanishes, so the solve's own rounding does not limit
/// the answer, only the residuals' does.
pub(crate) fn point_to_plane<Float: Real>(
    pairs: impl Iterator<Item = (Pair<Float>, Vector3<Float>)> + Clone,
) -> Result<Pose<Float>, Error> {
    let (centre, radius) = centre_and_radius(pairs.clone())?;
    let (mut hessian, gradient) = normal_equations(pairs, centre, radius);
    let mut trace = Float::ZERO;
    for (index, row) in hessian.iter().enumerate() {
        trace += row[index];
    }
    let damping = Float::of(DAMPING) * trace / Float::of(6.0);
    for (index, row) in hessian.iter_mut().enumerate() {
        row[index] += damping;
    }
    let step = solve_positive(hessian, gradient.map(|entry| -entry)).ok_or(Error::Degenerate)?;
    let [spin_x, spin_y, spin_z, shift_x, shift_y, shift_z] = step;
    let rotation =
        UnitQuaternion::from_scaled_axis(Vector3([spin_x, spin_y, spin_z]) * (Float::ONE / radius));
    let turned = rotation.matrix().map(|row| Vector3(row).dot(centre));
    Ok(Pose {
        rotation,
        translation: Vector3([shift_x, shift_y, shift_z]) + centre - Vector3(turned),
    })
}

fn centre_and_radius<Float: Real>(
    pairs: impl Iterator<Item = (Pair<Float>, Vector3<Float>)> + Clone,
) -> Result<(Vector3<Float>, Float), Error> {
    let mut count = 0;
    let mut total = Float::ZERO;
    let mut sum = Vector3::ZERO;
    for (pair, _) in pairs.clone() {
        count += usize::from(pair.weight > Float::ZERO);
        total += pair.weight;
        sum += pair.source * pair.weight;
    }
    if count < MINIMUM_PAIRS {
        return Err(Error::TooFewPairs {
            found: count,
            needed: MINIMUM_PAIRS,
        });
    }
    let centre = sum * (Float::ONE / total);
    let mut spread = Float::ZERO;
    for (pair, _) in pairs {
        spread += (pair.source - centre).squared_norm() * pair.weight;
    }
    let radius = (spread / total).root();
    if radius > Float::ZERO && radius.finite() {
        Ok((centre, radius))
    } else {
        Err(Error::Degenerate)
    }
}

#[expect(
    clippy::needless_range_loop,
    reason = "a matrix mirrored by its indices"
)]
fn normal_equations<Float: Real>(
    pairs: impl Iterator<Item = (Pair<Float>, Vector3<Float>)>,
    centre: Vector3<Float>,
    radius: Float,
) -> ([[Float; 6]; 6], [Float; 6]) {
    let mut hessian = [[Float::ZERO; 6]; 6];
    let mut gradient = [Float::ZERO; 6];
    let shrink = Float::ONE / radius;
    for (pair, normal) in pairs {
        let lever = (pair.source - centre) * shrink;
        let spin = lever.cross(normal);
        let jacobian = [spin[0], spin[1], spin[2], normal[0], normal[1], normal[2]];
        let residual = normal.dot(pair.source - pair.target) * pair.weight;
        for (row, entry) in jacobian.into_iter().enumerate() {
            gradient[row] += entry * residual;
            let weighted = entry * pair.weight;
            for (column, other) in jacobian[..=row].iter().enumerate() {
                hessian[row][column] += weighted * *other;
            }
        }
    }
    for row in 0..6 {
        for column in 0..row {
            hessian[column][row] = hessian[row][column];
        }
    }
    (hessian, gradient)
}
