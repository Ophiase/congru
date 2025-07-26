use point_cloud::{ PointCloud, Vec3 };
use rand_distr::{ Normal, Poisson };
use rand::distributions::Distribution;
use crate::sampler::PointCloudSampler;

/// Sample N_i points around each point using Gaussian distribution
pub fn sample_around_points(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    sigma: f64,
    mut sample_count_generator: impl FnMut(usize) -> usize
) -> PointCloud {
    let mut sampled_points = Vec::new();
    let normal_dist: Normal<f64> = Normal::new(0.0, sigma).unwrap();

    for (i, point) in original.points.iter().enumerate() {
        let n_samples = sample_count_generator(i);

        for _ in 0..n_samples {
            let noise = Vec3::new(
                normal_dist.sample(sampler.rng()),
                normal_dist.sample(sampler.rng()),
                normal_dist.sample(sampler.rng())
            );
            sampled_points.push(point + noise);
        }
    }

    PointCloud::from_points(sampled_points)
}

/// Sample exactly one point around each original point
pub fn sample_one_per_point(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    sigma: f64
) -> PointCloud {
    sample_around_points(original, sampler, sigma, |_| 1)
}

/// Sample using Poisson distribution for number of points per original point
pub fn sample_poisson_per_point(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    sigma: f64,
    lambda: f64
) -> PointCloud {
    let mut sampler_poisson = sampler.fork();
    let sampler_point = sampler;

    let poisson_dist = Poisson::new(lambda).unwrap();
    let sample_count_generator = move |_: usize| {
        poisson_dist.sample(sampler_poisson.rng()) as usize
    };

    sample_around_points(original, sampler_point, sigma, sample_count_generator)
}

/// Sample with fixed number of points per original point
pub fn sample_fixed_per_point(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    sigma: f64,
    n_per_point: usize
) -> PointCloud {
    sample_around_points(original, sampler, sigma, |_| n_per_point)
}

/// Sample with different sigma for each point
pub fn sample_with_varying_sigma(
    original: &PointCloud,
    sampler: &mut PointCloudSampler,
    sigma_generator: impl Fn(usize, &Vec3) -> f64,
    mut sample_count_generator: impl FnMut(usize) -> usize
) -> PointCloud {
    let mut sampled_points = Vec::new();

    for (i, point) in original.points.iter().enumerate() {
        let sigma = sigma_generator(i, point);
        let n_samples = sample_count_generator(i);
        let normal_dist = Normal::new(0.0, sigma).unwrap();

        for _ in 0..n_samples {
            let noise = Vec3::new(
                normal_dist.sample(sampler.rng()),
                normal_dist.sample(sampler.rng()),
                normal_dist.sample(sampler.rng())
            );
            sampled_points.push(point + noise);
        }
    }

    PointCloud::from_points(sampled_points)
}
