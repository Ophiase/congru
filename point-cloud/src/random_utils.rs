use crate::types::Vec3;
use rand::prelude::*;

/// Generate random point in unit sphere, then scale
pub fn random_sphere_point(radius: f64, rng: &mut ThreadRng) -> Vec3 {
    let u: f64 = rng.random_range(0.0..1.0);
    let r = radius * u.cbrt(); // Cube root for uniform distribution in volume

    let direction = super::linalg::random_unit_vector(rng);
    direction * r
}

/// Generate random point on sphere surface
pub fn random_sphere_surface_point(radius: f64, rng: &mut ThreadRng) -> Vec3 {
    let direction = super::linalg::random_unit_vector(rng);
    direction * radius
}
