use std::f64::consts::TAU;

use congru::RigidTransform;
use congru_synth::sequence::Handheld;
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
    /// The slow, wide wander of a camera held in a hand.
    pub handheld: Handheld,
    /// Two abrupt turns of the table, a third and two thirds of the way.
    pub knock_degrees: f64,
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
        let knocks = [self.frames / 3, 2 * self.frames / 3]
            .iter()
            .filter(|knock| frame >= **knock)
            .count();
        let knocked = if knocks == 2 {
            2.0
        } else {
            f64::from(u8::from(knocks == 1))
        };
        let spin =
            TAU * self.turns * self.progress(frame) + knocked * self.knock_degrees.to_radians();
        RigidTransform::from_axis_angle(Vector3::z() * spin, [0.0; 3]).then(&self.mount(frame))
    }

    /// Where the room, still in the world, appears at `frame`: the camera's
    /// own motion from where it stands at rest.
    #[must_use]
    pub fn room(&self, frame: usize) -> RigidTransform {
        Self::mount_at_rest().inverse().then(&self.mount(frame))
    }

    fn progress(&self, frame: usize) -> f64 {
        congru_vision::pixel::coordinate(frame)
            / congru_vision::pixel::coordinate(self.frames.max(1))
    }

    fn mount_at_rest() -> RigidTransform {
        Self::looking(Vector3::new(0.0, 0.04, DISTANCE + 0.04))
    }

    /// The turntable seen from the camera: fixed, but for the camera's sway,
    /// its wander in a hand and its jolts.
    fn mount(&self, frame: usize) -> RigidTransform {
        let progress = self.progress(frame);
        let sway = Vector3::new(
            0.03 * (TAU * progress).sin(),
            0.04,
            DISTANCE + 0.04 * (TAU * progress * 2.0).cos(),
        );
        Self::looking(sway)
            .then(&self.handheld.pose(frame))
            .then(&self.jolt(frame))
    }

    fn looking(position: Vector3<f64>) -> RigidTransform {
        let upright = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), TAU / 4.0);
        let look_down =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), LOOK_DOWN_DEGREES.to_radians());
        RigidTransform::from_parts(look_down * upright, position)
    }
}
