use point_cloud::{PointCloud, Vec3, Mat3, linalg};
use crate::sampler::PointCloudSampler;
use rand::prelude::*;

/// Apply a random rigid transformation (rotation + translation)
pub fn apply_random_transformation(
    point_cloud: &mut PointCloud,
    sampler: &mut PointCloudSampler,
    translation_bounds: f64,
) -> (Mat3, Vec3) {
    let rotation = generate_random_rotation(sampler);
    let translation = generate_random_translation(sampler, translation_bounds);
    
    point_cloud.transform(&rotation, &translation);
    (rotation, translation)
}

/// Generate a random rotation matrix
pub fn generate_random_rotation(sampler: &mut PointCloudSampler) -> Mat3 {
    // Generate random axis
    let axis = generate_random_unit_vector(sampler);
    
    // Generate random angle
    let angle: f64 = sampler.rng().gen_range(0.0..2.0 * std::f64::consts::PI);
    
    linalg::rotation_matrix_from_axis_angle(&axis, angle)
}

/// Generate a random translation vector within bounds
pub fn generate_random_translation(
    sampler: &mut PointCloudSampler,
    bounds: f64,
) -> Vec3 {
    Vec3::new(
        sampler.rng().gen_range(-bounds..bounds),
        sampler.rng().gen_range(-bounds..bounds),
        sampler.rng().gen_range(-bounds..bounds),
    )
}

/// Apply a specific rotation and translation
pub fn apply_transformation(
    point_cloud: &mut PointCloud,
    rotation: &Mat3,
    translation: &Vec3,
) {
    point_cloud.transform(rotation, translation);
}

/// Generate a random unit vector
fn generate_random_unit_vector(sampler: &mut PointCloudSampler) -> Vec3 {
    // Marsaglia method for uniform distribution on sphere
    loop {
        let x: f64 = sampler.rng().gen_range(-1.0..1.0);
        let y: f64 = sampler.rng().gen_range(-1.0..1.0);
        let norm_sq = x * x + y * y;
        
        if norm_sq < 1.0 {
            let z = 1.0 - 2.0 * norm_sq;
            let factor = 2.0 * (1.0 - norm_sq).sqrt();
            return Vec3::new(factor * x, factor * y, z);
        }
    }
}
