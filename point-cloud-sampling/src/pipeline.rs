use point_cloud::{PointCloud, Vec3, Mat3};
use crate::{sampler::PointCloudSampler, gaussian, uniform_noise, transform};

/// Configuration for the complete sampling process
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub gaussian_sigma: f64,
    pub sample_count_mode: SampleCountMode,
    pub noise_config: Option<NoiseConfig>,
    pub transformation_bounds: f64,
}

#[derive(Debug, Clone)]
pub enum SampleCountMode {
    One,
    Fixed(usize),
    Poisson(f64),
    Custom(fn(usize) -> usize),
}

#[derive(Debug, Clone)]
pub struct NoiseConfig {
    pub noise_type: NoiseType,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub enum NoiseType {
    BoundingBox(Vec3, Vec3),
    Exterior(f64), // expansion_factor
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            gaussian_sigma: 0.1,
            sample_count_mode: SampleCountMode::One,
            noise_config: None,
            transformation_bounds: 1.0,
        }
    }
}

/// Apply the complete sampling pipeline to a point cloud
pub fn apply_sampling_pipeline(
    original: &PointCloud,
    config: &SamplingConfig,
    sampler: &mut PointCloudSampler,
) -> (PointCloud, Mat3, Vec3) {
    // Step 1: Gaussian sampling around original points
    let sampled = match &config.sample_count_mode {
        SampleCountMode::One => {
            gaussian::sample_one_per_point(original, sampler, config.gaussian_sigma)
        }
        SampleCountMode::Fixed(n) => {
            gaussian::sample_fixed_per_point(original, sampler, config.gaussian_sigma, *n)
        }
        SampleCountMode::Poisson(lambda) => {
            gaussian::sample_poisson_per_point(original, sampler, config.gaussian_sigma, *lambda)
        }
        SampleCountMode::Custom(generator) => {
            gaussian::sample_around_points(original, sampler, config.gaussian_sigma, generator)
        }
    };

    // Step 2: Add uniform noise if configured
    let mut final_cloud = sampled;
    if let Some(noise_config) = &config.noise_config {
        let noise_cloud = match &noise_config.noise_type {
            NoiseType::BoundingBox(min, max) => {
                uniform_noise::add_uniform_noise(sampler, *min, *max, noise_config.count)
            }
            NoiseType::Exterior(expansion_factor) => {
                uniform_noise::add_exterior_uniform_noise(original, sampler, *expansion_factor, noise_config.count)
            }
        };
        final_cloud.merge(&noise_cloud);
    }

    // Step 3: Apply random transformation
    let (rotation, translation) = transform::apply_random_transformation(
        &mut final_cloud,
        sampler,
        config.transformation_bounds,
    );

    (final_cloud, rotation, translation)
}
