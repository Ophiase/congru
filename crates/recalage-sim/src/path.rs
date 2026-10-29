use std::f64::consts::TAU;

use nalgebra::{UnitQuaternion, Vector3};
use recalage::RigidTransform;

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
}

impl Turntable {
    #[must_use]
    pub fn pose(&self, frame: usize) -> RigidTransform {
        let progress = recalage_vision::pixel::coordinate(frame)
            / recalage_vision::pixel::coordinate(self.frames.max(1));
        let spin = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), TAU * self.turns * progress);
        let upright = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), TAU / 4.0);
        let look_down =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), LOOK_DOWN_DEGREES.to_radians());
        let sway = Vector3::new(
            0.03 * (TAU * progress).sin(),
            0.04,
            DISTANCE + 0.04 * (TAU * progress * 2.0).cos(),
        );
        RigidTransform::from_parts(look_down * upright * spin, sway)
    }
}
