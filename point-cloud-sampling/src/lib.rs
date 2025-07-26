pub mod sampler;
pub mod gaussian;
pub mod uniform_noise;
pub mod transform;
pub mod pipeline;

// Re-export main types and functions for convenience
pub use sampler::PointCloudSampler;
pub use pipeline::{SamplingConfig, SampleCountMode, NoiseConfig, NoiseType, apply_sampling_pipeline};
