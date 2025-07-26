use point_cloud::{ PointCloud, Vec3 };
use point_cloud_sampling::{ sampler::PointCloudSampler, uniform_noise };

#[test]
fn test_add_uniform_noise_basic() {
    let mut sampler = PointCloudSampler::with_seed(42);
    let min_bounds = Vec3::new(-1.0, -1.0, -1.0);
    let max_bounds = Vec3::new(1.0, 1.0, 1.0);
    
    let noise_cloud = uniform_noise::add_uniform_noise(&mut sampler, min_bounds, max_bounds, 10);
    
    // Should have exactly 10 points
    assert_eq!(noise_cloud.len(), 10);
}

#[test]
fn test_add_uniform_noise_zero_points() {
    let mut sampler = PointCloudSampler::with_seed(42);
    let min_bounds = Vec3::new(0.0, 0.0, 0.0);
    let max_bounds = Vec3::new(1.0, 1.0, 1.0);
    
    let noise_cloud = uniform_noise::add_uniform_noise(&mut sampler, min_bounds, max_bounds, 0);
    
    // Should have no points
    assert_eq!(noise_cloud.len(), 0);
}

#[test]
fn test_uniform_noise_within_bounds() {
    let mut sampler = PointCloudSampler::with_seed(42);
    let min_bounds = Vec3::new(2.0, 3.0, 4.0);
    let max_bounds = Vec3::new(5.0, 7.0, 9.0);
    
    let noise_cloud = uniform_noise::add_uniform_noise(&mut sampler, min_bounds, max_bounds, 50);
    
    // All points should be within bounds
    for point in &noise_cloud.points {
        assert!(point.x >= min_bounds.x && point.x <= max_bounds.x, 
               "X coordinate {} not in range [{}, {}]", point.x, min_bounds.x, max_bounds.x);
        assert!(point.y >= min_bounds.y && point.y <= max_bounds.y,
               "Y coordinate {} not in range [{}, {}]", point.y, min_bounds.y, max_bounds.y);
        assert!(point.z >= min_bounds.z && point.z <= max_bounds.z,
               "Z coordinate {} not in range [{}, {}]", point.z, min_bounds.z, max_bounds.z);
    }
}

#[test]
fn test_uniform_noise_reproducible() {
    let min_bounds = Vec3::new(0.0, 0.0, 0.0);
    let max_bounds = Vec3::new(1.0, 1.0, 1.0);
    
    let mut sampler1 = PointCloudSampler::with_seed(123);
    let mut sampler2 = PointCloudSampler::with_seed(123);
    
    let noise1 = uniform_noise::add_uniform_noise(&mut sampler1, min_bounds, max_bounds, 5);
    let noise2 = uniform_noise::add_uniform_noise(&mut sampler2, min_bounds, max_bounds, 5);
    
    // Same seed should produce same results
    assert_eq!(noise1.points, noise2.points);
}

#[test]
fn test_uniform_noise_different_seeds() {
    let min_bounds = Vec3::new(-2.0, -2.0, -2.0);
    let max_bounds = Vec3::new(2.0, 2.0, 2.0);
    
    let mut sampler1 = PointCloudSampler::with_seed(100);
    let mut sampler2 = PointCloudSampler::with_seed(200);
    
    let noise1 = uniform_noise::add_uniform_noise(&mut sampler1, min_bounds, max_bounds, 10);
    let noise2 = uniform_noise::add_uniform_noise(&mut sampler2, min_bounds, max_bounds, 10);
    
    // Different seeds should produce different results
    assert_ne!(noise1.points, noise2.points);
}

#[test]
fn test_add_random_count_uniform_noise() {
    let mut sampler = PointCloudSampler::with_seed(42);
    let min_bounds = Vec3::new(0.0, 0.0, 0.0);
    let max_bounds = Vec3::new(1.0, 1.0, 1.0);
    
    // Generator that returns fixed count
    let count_generator = || 7;
    
    let noise_cloud = uniform_noise::add_random_count_uniform_noise(
        &mut sampler, 
        min_bounds, 
        max_bounds, 
        count_generator
    );
    
    assert_eq!(noise_cloud.len(), 7);
}

#[test]
fn test_add_exterior_uniform_noise_basic() {
    let original = PointCloud::from_points(vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
    ]);
    
    let mut sampler = PointCloudSampler::with_seed(42);
    let exterior_noise = uniform_noise::add_exterior_uniform_noise(
        &original, 
        &mut sampler, 
        2.0, // expansion factor
        20   // num points
    );
    
    // Should generate some points (might be less than requested due to rejection sampling)
    assert!(exterior_noise.len() > 0);
    assert!(exterior_noise.len() <= 20);
}

#[test]
fn test_add_exterior_uniform_noise_empty_original() {
    let original = PointCloud::new();
    let mut sampler = PointCloudSampler::with_seed(42);
    
    let exterior_noise = uniform_noise::add_exterior_uniform_noise(
        &original, 
        &mut sampler, 
        2.0, 
        10
    );
    
    // Should return empty cloud for empty original
    assert_eq!(exterior_noise.len(), 0);
}

#[test]
fn test_exterior_noise_outside_original_bounds() {
    let original = PointCloud::from_points(vec![
        Vec3::new(2.0, 2.0, 2.0),
        Vec3::new(3.0, 3.0, 3.0),
    ]);
    
    let mut sampler = PointCloudSampler::with_seed(12345);
    let exterior_noise = uniform_noise::add_exterior_uniform_noise(
        &original, 
        &mut sampler, 
        3.0, // large expansion factor
        100  // many attempts
    );
    
    if let Some((orig_min, orig_max)) = original.bounding_box() {
        // Check that all exterior points are actually outside the original bounding box
        for point in &exterior_noise.points {
            let outside = point.x < orig_min.x || point.x > orig_max.x ||
                         point.y < orig_min.y || point.y > orig_max.y ||
                         point.z < orig_min.z || point.z > orig_max.z;
            
            assert!(outside, "Point {:?} should be outside original bounds [{:?}, {:?}]", 
                   point, orig_min, orig_max);
        }
    }
}

#[test]
fn test_uniform_distribution_properties() {
    let mut sampler = PointCloudSampler::with_seed(54321);
    let min_bounds = Vec3::new(0.0, 0.0, 0.0);
    let max_bounds = Vec3::new(10.0, 10.0, 10.0);
    
    // Generate many points to test statistical properties
    let noise_cloud = uniform_noise::add_uniform_noise(&mut sampler, min_bounds, max_bounds, 1000);
    
    // Calculate sample means (should be close to midpoint of bounds)
    let centroid = noise_cloud.centroid();
    let expected_center = (min_bounds + max_bounds) * 0.5;
    
    // Each coordinate mean should be close to expected center
    let tolerance = 0.5; // Allow some statistical variation
    assert!((centroid.x - expected_center.x).abs() < tolerance,
           "X mean {} not close to expected {}", centroid.x, expected_center.x);
    assert!((centroid.y - expected_center.y).abs() < tolerance,
           "Y mean {} not close to expected {}", centroid.y, expected_center.y);
    assert!((centroid.z - expected_center.z).abs() < tolerance,
           "Z mean {} not close to expected {}", centroid.z, expected_center.z);
}

#[test]
fn test_uniform_noise_coverage() {
    let mut sampler = PointCloudSampler::with_seed(999);
    let min_bounds = Vec3::new(0.0, 0.0, 0.0);
    let max_bounds = Vec3::new(2.0, 2.0, 2.0);
    
    // Generate many points
    let noise_cloud = uniform_noise::add_uniform_noise(&mut sampler, min_bounds, max_bounds, 100);
    
    // Check that we have good coverage across the space
    // Divide each axis into halves and check that we have points in different regions
    let mut low_x = 0;
    let mut high_x = 0;
    let mut low_y = 0;
    let mut high_y = 0;
    let mut low_z = 0;
    let mut high_z = 0;
    
    for point in &noise_cloud.points {
        if point.x < 1.0 { low_x += 1; } else { high_x += 1; }
        if point.y < 1.0 { low_y += 1; } else { high_y += 1; }
        if point.z < 1.0 { low_z += 1; } else { high_z += 1; }
    }
    
    // Each half should have at least some points (not a strict requirement, but likely with 100 points)
    assert!(low_x > 0 && high_x > 0, "Poor X coverage: low={}, high={}", low_x, high_x);
    assert!(low_y > 0 && high_y > 0, "Poor Y coverage: low={}, high={}", low_y, high_y);
    assert!(low_z > 0 && high_z > 0, "Poor Z coverage: low={}, high={}", low_z, high_z);
}
