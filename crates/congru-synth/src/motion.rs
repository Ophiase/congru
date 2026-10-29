use std::f64::consts::PI;

use nalgebra::{Unit, UnitQuaternion, Vector3};
use rand::{Rng, RngExt};
use congru::RigidTransform;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionBounds {
    /// Largest rotation angle, in radians; `π` allows any rotation.
    pub max_angle: f64,
    pub max_translation: f64,
}

impl MotionBounds {
    #[must_use]
    pub const fn any_rotation(max_translation: f64) -> Self {
        Self {
            max_angle: PI,
            max_translation,
        }
    }

    /// An axis uniform on the sphere, an angle uniform below the bound, a
    /// translation uniform in the ball.
    pub fn draw(&self, random: &mut impl Rng) -> RigidTransform {
        let axis = unit_vector(random);
        let angle = random.random_range(0.0..=self.max_angle);
        let length = self.max_translation * random.random::<f64>().cbrt();
        RigidTransform::from_parts(
            UnitQuaternion::from_axis_angle(&axis, angle),
            unit_vector(random).into_inner() * length,
        )
    }
}

pub(crate) fn unit_vector(random: &mut impl Rng) -> Unit<Vector3<f64>> {
    loop {
        let candidate = Vector3::from_fn(|_, _| random.random_range(-1.0..=1.0));
        let length = candidate.norm();
        if length > 1e-3 && length <= 1.0 {
            return Unit::new_unchecked(candidate / length);
        }
    }
}
