use point_cloud::{generation, PointCloud, Vec3};
use point_cloud_sampling::{*, gaussian, uniform_noise, transform, pipeline};

fn main() {
    println!("=== Point Cloud Sampling Library Demo ===\n");

    // Create an original point cloud
    let original = generation::sphere_surface(Vec3::new(0.0, 0.0, 0.0), 2.0, 15);
    println!("Original point cloud: {} points", original.len());
    println!("Original centroid: {:?}", original.centroid());
    if let Some((min, max)) = original.bounding_box() {
        println!("Original bounding box: min={:?}, max={:?}", min, max);
    }
    println!();

    let mut sampler = PointCloudSampler::new();

    // Demo 1: Basic Gaussian sampling
    println!("1. Gaussian Sampling:");
    
    // One point per original point
    let sampled_one = gaussian::sample_one_per_point(&original, &mut sampler, 0.1);
    println!("   One per point: {} points", sampled_one.len());
    
    // Fixed number per original point
    let sampled_fixed = gaussian::sample_fixed_per_point(&original, &mut sampler, 0.1, 3);
    println!("   Fixed 3 per point: {} points", sampled_fixed.len());
    
    // Poisson distribution
    let sampled_poisson = gaussian::sample_poisson_per_point(&original, &mut sampler, 0.1, 2.5);
    println!("   Poisson(λ=2.5) per point: {} points", sampled_poisson.len());
    println!();

    // Demo 2: Uniform noise addition
    println!("2. Uniform Noise Addition:");
    
    // Basic uniform noise in bounding box
    let noise_box = uniform_noise::add_uniform_noise(
        &mut sampler,
        Vec3::new(-3.0, -3.0, -3.0),
        Vec3::new(3.0, 3.0, 3.0),
        20
    );
    println!("   Added {} uniform noise points in box", noise_box.len());
    
    // Exterior noise (outside original bounding box)
    let noise_exterior = uniform_noise::add_exterior_uniform_noise(
        &original,
        &mut sampler,
        1.5, // expansion factor
        15
    );
    println!("   Added {} exterior noise points", noise_exterior.len());
    println!();

    // Demo 3: Transformations
    println!("3. Rigid Transformations:");
    
    let mut transform_test = original.clone();
    let original_centroid = transform_test.centroid();
    
    let (rotation, translation) = transform::apply_random_transformation(
        &mut transform_test,
        &mut sampler,
        2.0
    );
    
    let new_centroid = transform_test.centroid();
    println!("   Original centroid: {:?}", original_centroid);
    println!("   After transformation: {:?}", new_centroid);
    println!("   Translation applied: {:?}", translation);
    println!("   Rotation determinant: {:.6}", rotation.determinant());
    println!();

    // Demo 4: Complete sampling pipeline - Variant 1 (Simplest)
    println!("4. Complete Pipeline - Variant 1 (Simplest):");
    
    let config_simple = SamplingConfig {
        gaussian_sigma: 0.05,
        sample_count_mode: SampleCountMode::One,
        noise_config: None, // No noise in simplest case
        transformation_bounds: 1.5,
    };
    
    let (result_simple, rot_simple, trans_simple) = apply_sampling_pipeline(
        &original,
        &config_simple,
        &mut sampler
    );
    
    println!("   Input: {} points -> Output: {} points", original.len(), result_simple.len());
    println!("   Applied rotation determinant: {:.6}", rot_simple.determinant());
    println!("   Applied translation: {:?}", trans_simple);
    println!();

    // Demo 5: Complete pipeline - Variant 2 (With noise)
    println!("5. Complete Pipeline - Variant 2 (With bounded noise):");
    
    let config_noise = SamplingConfig {
        gaussian_sigma: 0.08,
        sample_count_mode: SampleCountMode::Fixed(2),
        noise_config: Some(NoiseConfig {
            noise_type: NoiseType::Exterior(1.8),
            count: 10,
        }),
        transformation_bounds: 2.0,
    };
    
    let (result_noise, rot_noise, trans_noise) = apply_sampling_pipeline(
        &original,
        &config_noise,
        &mut sampler
    );
    
    println!("   Input: {} points -> Output: {} points", original.len(), result_noise.len());
    println!("   Expected: {} sampled + {} noise = {} total", 
             original.len() * 2, 10, original.len() * 2 + 10);
    println!("   Applied rotation determinant: {:.6}", rot_noise.determinant());
    println!("   Applied translation: {:?}", trans_noise);
    println!();

    // Demo 6: Complete pipeline - Variant 3 (General case with Poisson)
    println!("6. Complete Pipeline - Variant 3 (General case):");
    
    let config_general = SamplingConfig {
        gaussian_sigma: 0.12,
        sample_count_mode: SampleCountMode::Poisson(3.0),
        noise_config: Some(NoiseConfig {
            noise_type: NoiseType::BoundingBox(
                Vec3::new(-4.0, -4.0, -4.0),
                Vec3::new(4.0, 4.0, 4.0)
            ),
            count: 25,
        }),
        transformation_bounds: 3.0,
    };
    
    let (result_general, rot_general, trans_general) = apply_sampling_pipeline(
        &original,
        &config_general,
        &mut sampler
    );
    
    println!("   Input: {} points -> Output: {} points", original.len(), result_general.len());
    println!("   Poisson sampling + 25 uniform noise points");
    println!("   Applied rotation determinant: {:.6}", rot_general.determinant());
    println!("   Applied translation: {:?}", trans_general);
    
    if let Some((min, max)) = result_general.bounding_box() {
        println!("   Final bounding box: min={:?}, max={:?}", min, max);
    }
    println!();

    // Demo 7: Statistical analysis
    println!("7. Statistical Analysis:");
    println!("   Comparing original vs final point clouds:");
    
    let original_stats = compute_basic_stats(&original);
    let final_stats = compute_basic_stats(&result_general);
    
    println!("   Original - Count: {}, Spread: {:.3}", 
             original_stats.count, original_stats.spread);
    println!("   Final    - Count: {}, Spread: {:.3}", 
             final_stats.count, final_stats.spread);
    println!();

    println!("=== Demo Complete ===");
    println!("The sampling pipeline successfully demonstrates:");
    println!("- Gaussian sampling around original points");
    println!("- Uniform noise addition (interior/exterior)");
    println!("- Random rigid transformations");
    println!("- Complete pipeline for all problem variants");
}

struct BasicStats {
    count: usize,
    spread: f64,
}

fn compute_basic_stats(pc: &PointCloud) -> BasicStats {
    let spread = if let Some((min, max)) = pc.bounding_box() {
        (max - min).norm()
    } else {
        0.0
    };
    
    BasicStats {
        count: pc.len(),
        spread,
    }
}
