use congru::{PointCloud, RigidTransform, Vector};
use congru_embedded::{Pose, Real, UnitQuaternion, Vector3};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

/// What rounding alone may leave. A coordinate of magnitude `m` is known to
/// `m ε / 2`; a pose composes a dozen such operations per point, and the fits
/// average them, so a few tens of `ε m` are the most an answer can lose.
pub const ROUNDING_STEPS: f64 = 64.0;

/// The rotation rounding may leave on coordinates of magnitude `magnitude`
/// spread over `spread`: an offset error over its lever.
pub fn rotation_floor<Float: Real>(magnitude: f64, spread: f64) -> f64 {
    ROUNDING_STEPS * Float::EPSILON.as_f64() * (magnitude / spread).max(1.0)
}

/// The translation rounding may leave: an offset error, plus the rotation's
/// error over the coordinates' lever.
pub fn translation_floor<Float: Real>(magnitude: f64, spread: f64) -> f64 {
    ROUNDING_STEPS * Float::EPSILON.as_f64() * magnitude * (magnitude / spread).max(1.0)
}

pub fn cast<Float: Real>(point: [f64; 3]) -> [Float; 3] {
    point.map(Float::of)
}

pub fn cloud_of<Float: Real>(cloud: &PointCloud) -> Vec<[Float; 3]> {
    cloud
        .points()
        .iter()
        .map(|point| cast([point.x, point.y, point.z]))
        .collect()
}

pub fn pose_of<Float: Real>(transform: &RigidTransform) -> Pose<Float> {
    let rotation = transform.rotation();
    let translation = transform.translation();
    Pose {
        rotation: UnitQuaternion::normalised(
            [rotation.w, rotation.i, rotation.j, rotation.k].map(Float::of),
        ),
        translation: Vector3(cast([translation.x, translation.y, translation.z])),
    }
}

pub fn raised<Float: Real>(pose: &Pose<Float>) -> Pose<f64> {
    Pose {
        rotation: UnitQuaternion::normalised(pose.rotation.components().map(Real::as_f64)),
        translation: Vector3(pose.translation.0.map(Real::as_f64)),
    }
}

/// Rotation and translation errors, in `f64` whatever the pose's scalar.
pub fn errors<Float: Real>(found: &Pose<Float>, truth: &RigidTransform) -> (f64, f64) {
    let found = raised(found);
    let truth = pose_of::<f64>(truth);
    (
        found.rotation_error_to(&truth),
        found.translation_error_to(&truth),
    )
}

/// A cloud uniform in a cube of half-side `half`, about `centre`.
pub fn cube(count: usize, half: f64, centre: [f64; 3], seed: u64) -> Vec<[f64; 3]> {
    let mut random = ChaCha8Rng::seed_from_u64(seed);
    (0..count)
        .map(|_| centre.map(|coordinate| coordinate + half * (random.random::<f64>() * 2.0 - 1.0)))
        .collect()
}

pub fn motion() -> RigidTransform {
    RigidTransform::from_axis_angle(Vector::new(0.7, -0.2, 1.1), [0.1, 0.05, -0.2])
}

pub fn moved(motion: &RigidTransform, point: [f64; 3]) -> [f64; 3] {
    let turned = motion.rotation() * Vector::from(point) + motion.translation();
    [turned.x, turned.y, turned.z]
}
