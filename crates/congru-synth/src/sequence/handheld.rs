use std::f64::consts::TAU;

use congru::RigidTransform;
use nalgebra::{UnitQuaternion, Vector3};

use crate::casts::as_float;

const OCTAVES: usize = 4;
/// Fixed phases, one per octave and axis, far from any simple fraction of
/// a turn, so the six motions never fall into step.
const PHASES: [[f64; 6]; OCTAVES] = [
    [0.13, 0.71, 0.37, 0.93, 0.29, 0.57],
    [0.83, 0.19, 0.61, 0.07, 0.47, 0.31],
    [0.41, 0.97, 0.23, 0.67, 0.11, 0.79],
    [0.59, 0.03, 0.89, 0.43, 0.73, 0.17],
];

/// A camera held in a hand: a slow, wide wander of its rotation and
/// position. Each of the six motions is smooth noise, a sum of sines whose
/// amplitude halves as their frequency doubles, the slowest of
/// `period` frames; deterministic, it depends on the frame number only.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Handheld {
    /// Largest rotation away from rest, in radians.
    pub angle: f64,
    /// Largest shift away from rest.
    pub shift: f64,
    /// Frames of the slowest octave.
    pub period: f64,
}

impl Handheld {
    /// The camera's wander at `frame`, as a motion of the scene about the
    /// camera.
    #[must_use]
    pub fn pose(&self, frame: usize) -> RigidTransform {
        if self.period <= 0.0 {
            return RigidTransform::identity();
        }
        let time = as_float(frame) / self.period;
        let wander = |axis: usize| {
            let (sum, weight) = (0..OCTAVES).fold((0.0, 0.0), |(sum, weight), octave| {
                let amplitude = 0.5_f64.powi(i32::try_from(octave).unwrap_or(0));
                let frequency = 2.0_f64.powi(i32::try_from(octave).unwrap_or(0));
                let wave = (TAU * (frequency * time + PHASES[octave][axis])).sin();
                (sum + amplitude * wave, weight + amplitude)
            });
            sum / weight
        };
        let turn = Vector3::new(wander(0), wander(1), wander(2)) * self.angle;
        let shift = Vector3::new(wander(3), wander(4), wander(5)) * self.shift;
        RigidTransform::from_parts(UnitQuaternion::from_scaled_axis(turn), shift)
    }
}
