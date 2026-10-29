use std::f64::consts::TAU;

use nalgebra::{UnitQuaternion, Vector3};
use congru::RigidTransform;

use crate::casts::as_float;

/// The smooth part of the motion: the model, `distance` in front of the
/// sensor, turns about a slowly wandering axis and sways around its place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drift {
    pub distance: f64,
    /// Full turns over the sequence.
    pub turns: f64,
    /// Largest sideways sway.
    pub sway: f64,
}

/// An abrupt change of pose at `frame`, kept for the rest of the sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jump {
    pub frame: usize,
    /// Rotation vector, in radians.
    pub turn: [f64; 3],
    pub shift: [f64; 3],
}

impl Drift {
    pub(super) fn pose(&self, frame: usize, jumps: &[Jump]) -> RigidTransform {
        let time = as_float(frame) / 60.0;
        let axis =
            Vector3::new(0.3 * (0.7 * time).sin(), 1.0, 0.25 * (0.5 * time).cos()).normalize();
        let angle = TAU * self.turns * time;
        let smooth = RigidTransform::from_parts(
            UnitQuaternion::from_scaled_axis(axis * angle),
            [
                self.sway * (0.9 * time).sin(),
                0.5 * self.sway * (1.3 * time).cos(),
                self.distance,
            ],
        );
        let knocked = jumps.iter().filter(|jump| jump.frame <= frame).fold(
            RigidTransform::identity(),
            |total, jump| {
                total.then(&RigidTransform::from_axis_angle(
                    jump.turn.into(),
                    jump.shift,
                ))
            },
        );
        let place = smooth.translation();
        let about_place = RigidTransform::from_parts(UnitQuaternion::identity(), -place)
            .then(&knocked)
            .then(&RigidTransform::from_parts(
                UnitQuaternion::identity(),
                place,
            ));
        smooth.then(&about_place)
    }
}
