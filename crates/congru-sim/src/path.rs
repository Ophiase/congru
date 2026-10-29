use std::f64::consts::TAU;

use congru::RigidTransform;
use nalgebra::{UnitQuaternion, Vector3};

const DISTANCE: f64 = 0.42;
const LOOK_DOWN_DEGREES: f64 = 28.0;

/// The part on a turntable in front of the camera: it turns about its own
/// vertical axis while the camera, slightly above, looks down at it. Poses
/// map the part's frame (z up) to the camera's (y down, z forward).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Turntable {
    pub frames: usize,
    /// Full turns over the whole sequence.
    pub turns: f64,
    /// A jolt of the camera every frame: a rotation about it of this many
    /// degrees at most, on an axis that changes from frame to frame.
    pub shake_degrees: f64,
}

impl Turntable {
    /// Deterministic: a frame's jolt depends on its number only.
    fn jolt(&self, frame: usize) -> RigidTransform {
        let phase = congru_vision::pixel::coordinate(frame);
        let axis = Vector3::new(
            (phase * 2.39).sin(),
            (phase * 1.71).cos(),
            (phase * 3.13).sin(),
        );
        let size = (phase * 7.77).sin() * self.shake_degrees.to_radians();
        RigidTransform::from_parts(
            UnitQuaternion::from_scaled_axis(
                axis.try_normalize(f64::EPSILON).unwrap_or_else(Vector3::x) * size,
            ),
            [0.0; 3],
        )
    }

    #[must_use]
    pub fn pose(&self, frame: usize) -> RigidTransform {
        let progress = congru_vision::pixel::coordinate(frame)
            / congru_vision::pixel::coordinate(self.frames.max(1));
        let spin = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), TAU * self.turns * progress);
        let upright = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), TAU / 4.0);
        let look_down =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), LOOK_DOWN_DEGREES.to_radians());
        let sway = Vector3::new(
            0.03 * (TAU * progress).sin(),
            0.04,
            DISTANCE + 0.04 * (TAU * progress * 2.0).cos(),
        );
        RigidTransform::from_parts(look_down * upright * spin, sway).then(&self.jolt(frame))
    }
}
