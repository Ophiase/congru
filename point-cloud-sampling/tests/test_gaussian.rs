use point_cloud::{ PointCloud, Vec3 };
use point_cloud_sampling::{ sampler::PointCloudSampler, gaussian };

#[test]
fn test_sample_one_per_point_basic() {
    let original = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)]
    );

    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_one_per_point(&original, &mut sampler, 0.1);

    // Should have same number of points as original
    assert_eq!(sampled.len(), original.len());
}

#[test]
fn test_sample_one_per_point_empty_cloud() {
    let original = PointCloud::new();
    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_one_per_point(&original, &mut sampler, 0.1);

    // Should remain empty
    assert_eq!(sampled.len(), 0);
}

#[test]
fn test_sample_one_per_point_reproducible() {
    let original = PointCloud::from_points(
        vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]
    );

    let mut sampler1 = PointCloudSampler::with_seed(123);
    let mut sampler2 = PointCloudSampler::with_seed(123);

    let sampled1 = gaussian::sample_one_per_point(&original, &mut sampler1, 0.2);
    let sampled2 = gaussian::sample_one_per_point(&original, &mut sampler2, 0.2);

    // Same seed should produce same results
    assert_eq!(sampled1.points, sampled2.points);
}

#[test]
fn test_sample_one_per_point_different_seeds() {
    let original = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)]
    );

    let mut sampler1 = PointCloudSampler::with_seed(100);
    let mut sampler2 = PointCloudSampler::with_seed(200);

    let sampled1 = gaussian::sample_one_per_point(&original, &mut sampler1, 0.1);
    let sampled2 = gaussian::sample_one_per_point(&original, &mut sampler2, 0.1);

    // Different seeds should produce different results
    assert_ne!(sampled1.points, sampled2.points);
}

#[test]
fn test_sample_around_points_near_original() {
    let original = PointCloud::from_points(vec![Vec3::new(5.0, 5.0, 5.0)]);

    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_one_per_point(&original, &mut sampler, 0.1);

    // The sampled point should be close to the original (within reasonable bounds)
    let original_point = original.points[0];
    let sampled_point = sampled.points[0];
    let distance = (sampled_point - original_point).norm();

    // With sigma=0.1, distance should be within 3*sigma with high probability (99.7%)
    assert!(distance < 0.3, "Sampled point too far from original: distance = {}", distance);
}

#[test]
fn test_sample_fixed_per_point() {
    let original = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)]
    );

    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_fixed_per_point(&original, &mut sampler, 0.1, 3);

    // Should have 3 samples per original point
    assert_eq!(sampled.len(), original.len() * 3);
}

#[test]
fn test_sample_poisson_per_point() {
    let original = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)]
    );

    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_poisson_per_point(&original, &mut sampler, 0.1, 2.0);

    // Should have some points (Poisson with lambda=2.0 rarely gives 0)
    assert!(sampled.len() > 0);

    // With lambda=2.0 and 3 original points, we expect around 6 points total
    // Allow for some variance (between 1 and 15 points should be reasonable)
    assert!(sampled.len() >= 1 && sampled.len() <= 15);
}

#[test]
fn test_sample_with_varying_sigma() {
    let original = PointCloud::from_points(
        vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0)]
    );

    let mut sampler = PointCloudSampler::with_seed(42);

    // Use different sigma for each point
    let sigma_generator = |i: usize, _point: &Vec3| -> f64 {
        if i == 0 { 0.1 } else { 1.0 }
    };

    let sampled = gaussian::sample_with_varying_sigma(
        &original,
        &mut sampler,
        sigma_generator,
        |_| 1
    );

    assert_eq!(sampled.len(), 2);

    // First point should be close to original (small sigma)
    let dist1 = (sampled.points[0] - original.points[0]).norm();
    assert!(dist1 < 0.5);

    // Second point can be further (larger sigma) - but this is probabilistic
    // We'll just check it exists
    assert!(sampled.points.len() > 1);
}

#[test]
fn test_gaussian_distribution_properties() {
    let original = PointCloud::from_points(vec![Vec3::new(0.0, 0.0, 0.0)]);
    let mut sampler = PointCloudSampler::with_seed(12345);

    // Generate many samples to test statistical properties
    let sampled = gaussian::sample_fixed_per_point(&original, &mut sampler, 1.0, 1000);

    // Calculate sample mean (should be close to original point)
    let mean = sampled.centroid();
    let original_point = original.points[0];

    // Mean should be close to original point (within statistical noise)
    let mean_error = (mean - original_point).norm();
    assert!(mean_error < 0.1, "Sample mean too far from original: {}", mean_error);

    // Calculate sample variance in one dimension (should be close to sigma^2 = 1.0)
    let x_values: Vec<f64> = sampled.points
        .iter()
        .map(|p| p.x)
        .collect();
    let x_mean = x_values.iter().sum::<f64>() / (x_values.len() as f64);
    let x_variance =
        x_values
            .iter()
            .map(|&x| (x - x_mean).powi(2))
            .sum::<f64>() / ((x_values.len() - 1) as f64);

    // Variance should be close to sigma^2 = 1.0 (allow some statistical variation)
    assert!(
        x_variance > 0.8 && x_variance < 1.2,
        "Sample variance not close to expected: {}",
        x_variance
    );
}

#[test]
fn test_zero_sigma_gives_exact_copies() {
    let original = PointCloud::from_points(
        vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]
    );

    let mut sampler = PointCloudSampler::with_seed(42);
    let sampled = gaussian::sample_one_per_point(&original, &mut sampler, 0.0);

    // With sigma=0, sampled points should be exactly the same as original
    for (orig, samp) in original.points.iter().zip(sampled.points.iter()) {
        assert_eq!(*orig, *samp);
    }
}
