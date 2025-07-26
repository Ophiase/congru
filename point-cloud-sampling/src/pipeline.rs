use point_cloud::{ PointCloud, Vec3, Mat3 };
use crate::{ sampler::PointCloudSampler, gaussian, uniform_noise, transform };

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

fn apply_sample_step(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    config: &SamplingConfig
) -> PointCloud {
    match &config.sample_count_mode {
        SampleCountMode::One =>
            gaussian::sample_one_per_point(original, sampler, config.gaussian_sigma),
        SampleCountMode::Fixed(n) =>
            gaussian::sample_fixed_per_point(original, sampler, config.gaussian_sigma, *n),
        SampleCountMode::Poisson(lambda) =>
            gaussian::sample_poisson_per_point(original, sampler, config.gaussian_sigma, *lambda),
        SampleCountMode::Custom(generator) =>
            gaussian::sample_around_points(original, sampler, config.gaussian_sigma, generator),
    }
}

fn apply_uniform_noise_step(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    config: &SamplingConfig
) -> PointCloud {
    if let Some(noise_config) = &config.noise_config {
        match &noise_config.noise_type {
            NoiseType::BoundingBox(min, max) => {
                uniform_noise::add_uniform_noise(sampler, *min, *max, noise_config.count)
            }
            NoiseType::Exterior(expansion_factor) => {
                uniform_noise::add_exterior_uniform_noise(
                    original,
                    sampler,
                    *expansion_factor,
                    noise_config.count
                )
            }
        }
    } else {
        PointCloud::new() // No noise added
    }
}

fn apply_transformation_step(
    original: &mut PointCloud,
    sampler: &mut PointCloudSampler,
    config: &SamplingConfig
) -> (Mat3, Vec3) {
    transform::apply_random_transformation(original, sampler, config.transformation_bounds)
}

/// Apply the complete sampling pipeline to a point cloud
pub fn apply_sampling_pipeline(
    original: &PointCloud,
    config: &SamplingConfig,
    sampler: &mut PointCloudSampler
) -> (PointCloud, Mat3, Vec3) {
    // Step 1: Gaussian sampling around original points
    let sampled = apply_sample_step(original, sampler, config);
    // Step 2: Add uniform noise if configured
    let mut final_cloud = apply_uniform_noise_step(&sampled, sampler, config);
    // Step 3: Apply random transformation
    let (rotation, translation) = apply_transformation_step(&mut final_cloud, sampler, config);

    (final_cloud, rotation, translation)
}
