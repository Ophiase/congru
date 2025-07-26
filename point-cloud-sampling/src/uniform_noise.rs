use point_cloud::{ PointCloud, Vec3 };
use rand_distr::Uniform;
use rand::{ distributions::Distribution, Rng };
use crate::sampler::PointCloudSampler;

/// Add uniform noise points within a bounding box
pub fn add_uniform_noise(
    sampler: &mut PointCloudSampler,
    min_bounds: Vec3,
    max_bounds: Vec3,
    num_noise_points: usize
) -> PointCloud {
    let mut noise_points = Vec::with_capacity(num_noise_points);

    let x_dist = Uniform::new(min_bounds.x, max_bounds.x);
    let y_dist = Uniform::new(min_bounds.y, max_bounds.y);
    let z_dist = Uniform::new(min_bounds.z, max_bounds.z);

    for _ in 0..num_noise_points {
        let point = Vec3::new(
            x_dist.sample(sampler.rng()),
            y_dist.sample(sampler.rng()),
            z_dist.sample(sampler.rng())
        );
        noise_points.push(point);
    }

    PointCloud::from_points(noise_points)
}

/// Add uniform noise with random count
pub fn add_random_count_uniform_noise(
    sampler: &mut PointCloudSampler,
    min_bounds: Vec3,
    max_bounds: Vec3,
    noise_count_generator: impl Fn() -> usize
) -> PointCloud {
    let num_noise_points = noise_count_generator();
    add_uniform_noise(sampler, min_bounds, max_bounds, num_noise_points)
}

/// Add uniform noise outside the convex hull (simplified as expanded bounding box)
pub fn add_exterior_uniform_noise(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    expansion_factor: f64,
    num_noise_points: usize
) -> PointCloud {
    let has_bounds: bool = original.bounding_box().is_some();
    if !has_bounds {
        return PointCloud::new(); // Return empty cloud if no bounding box
    }

    let (min, max) = original.bounding_box().unwrap();
    let center = (min + max) * 0.5;
    let size = max - min;
    let expanded_size = size * expansion_factor;

    let expanded_min = center - expanded_size * 0.5;
    let expanded_max = center + expanded_size * 0.5;

    // Generate points in expanded box but outside original box
    let mut noise_points = Vec::new();
    let mut attempts = 0;
    let max_attempts: usize = num_noise_points * 10;

    while noise_points.len() < num_noise_points && attempts < max_attempts {
        let candidate = Vec3::new(
            sampler.rng().gen_range(expanded_min.x..expanded_max.x),
            sampler.rng().gen_range(expanded_min.y..expanded_max.y),
            sampler.rng().gen_range(expanded_min.z..expanded_max.z)
        );

        // Check if point is outside original bounding box
        if
            candidate.x < min.x ||
            candidate.x > max.x ||
            candidate.y < min.y ||
            candidate.y > max.y ||
            candidate.z < min.z ||
            candidate.z > max.z
        {
            noise_points.push(candidate);
        }

        attempts += 1;
    }

    PointCloud::from_points(noise_points)
}
