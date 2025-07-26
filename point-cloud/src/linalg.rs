use crate::types::{ Vec3, Mat3 };
use rand::{ prelude::*, rng };

/// Generate a random rotation matrix
pub fn random_rotation() -> Mat3 {
    let mut rng = rng();

    // Generate random axis
    let axis = random_unit_vector(&mut rng);

    // Generate random angle
    let angle: f64 = rng.random_range(0.0..2.0 * std::f64::consts::PI);

    rotation_matrix_from_axis_angle(&axis, angle)
}

/// Create rotation matrix from axis and angle
pub fn rotation_matrix_from_axis_angle(axis: &Vec3, angle: f64) -> Mat3 {
    let axis = axis.normalize();
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let one_minus_cos = 1.0 - cos_a;

    Mat3::new(
        cos_a + axis.x * axis.x * one_minus_cos,
        axis.x * axis.y * one_minus_cos - axis.z * sin_a,
        axis.x * axis.z * one_minus_cos + axis.y * sin_a,

        axis.y * axis.x * one_minus_cos + axis.z * sin_a,
        cos_a + axis.y * axis.y * one_minus_cos,
        axis.y * axis.z * one_minus_cos - axis.x * sin_a,

        axis.z * axis.x * one_minus_cos - axis.y * sin_a,
        axis.z * axis.y * one_minus_cos + axis.x * sin_a,
        cos_a + axis.z * axis.z * one_minus_cos
    )
}

/// Generate random translation vector
pub fn random_translation(bounds: f64) -> Vec3 {
    let mut rng = rng();
    Vec3::new(
        rng.random_range(-bounds..bounds),
        rng.random_range(-bounds..bounds),
        rng.random_range(-bounds..bounds)
    )
}

/// Generate random unit vector
pub fn random_unit_vector(rng: &mut ThreadRng) -> Vec3 {
    // Marsaglia method for uniform distribution on sphere
    loop {
        let x: f64 = rng.random_range(-1.0..1.0);
        let y: f64 = rng.random_range(-1.0..1.0);
        let norm_sq = x * x + y * y;

        if norm_sq < 1.0 {
            let z = 1.0 - 2.0 * norm_sq;
            let factor = 2.0 * (1.0 - norm_sq).sqrt();
            return Vec3::new(factor * x, factor * y, z);
        }
    }
}
