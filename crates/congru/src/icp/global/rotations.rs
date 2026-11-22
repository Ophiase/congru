use std::f64::consts::{SQRT_2, TAU};

use nalgebra::{Quaternion, UnitQuaternion};

use crate::geometry::count_as_float;

/// The second constant of super-Fibonacci spirals, the root of
/// `x⁴ = x + 4` (Alexa, CVPR 2022).
const SPIRAL: f64 = 1.533_751_168_755_204_3;

/// `count` rotations spread evenly over SO(3), deterministic.
pub(crate) fn spread_rotations(count: usize) -> Vec<UnitQuaternion<f64>> {
    let total = count_as_float(count);
    (0..count)
        .map(|index| {
            let rank = count_as_float(index) + 0.5;
            let inner = (rank / total).sqrt();
            let outer = (1.0 - rank / total).sqrt();
            let first_angle = TAU * rank / SQRT_2;
            let second_angle = TAU * rank / SPIRAL;
            UnitQuaternion::from_quaternion(Quaternion::new(
                outer * second_angle.cos(),
                inner * first_angle.sin(),
                inner * first_angle.cos(),
                outer * second_angle.sin(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNT: usize = 512;

    #[test]
    fn every_rotation_has_a_close_sample() {
        let samples = spread_rotations(COUNT);
        let probes = spread_rotations(COUNT / 7);
        let farthest = probes
            .iter()
            .map(|probe| {
                samples
                    .iter()
                    .map(|sample| sample.angle_to(probe))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max);
        let expected_spacing = (8.0 * std::f64::consts::PI.powi(2) / count_as_float(COUNT)).cbrt();
        assert!(
            farthest < 1.5 * expected_spacing,
            "{farthest} against {expected_spacing}"
        );
    }
}
