use point_cloud_sampling::sampler::PointCloudSampler;
use rand::{ RngCore };
use std::collections::HashSet;

#[test]
fn test_sampler_with_seed_reproducible() {
    let mut sampler1 = PointCloudSampler::with_seed(42);
    let mut sampler2 = PointCloudSampler::with_seed(42);

    // Same seed should produce same sequence of random numbers
    let val1: u64 = sampler1.rng().next_u64();
    let val2: u64 = sampler2.rng().next_u64();

    assert_eq!(val1, val2);
}

#[test]
fn test_sampler_different_seeds_different_output() {
    let mut sampler1 = PointCloudSampler::with_seed(42);
    let mut sampler2 = PointCloudSampler::with_seed(43);

    // Different seeds should produce different sequences
    let val1: u64 = sampler1.rng().next_u64();
    let val2: u64 = sampler2.rng().next_u64();

    assert_ne!(val1, val2);
}

#[test]
fn test_fork_creates_independent_samplers() {
    let mut original = PointCloudSampler::with_seed(42);
    let mut fork1 = original.fork();
    let mut fork2 = original.fork();

    // Generate some random values from each sampler
    let orig_vals: Vec<u64> = (0..10).map(|_| original.rng().next_u64()).collect();
    let fork1_vals: Vec<u64> = (0..10).map(|_| fork1.rng().next_u64()).collect();
    let fork2_vals: Vec<u64> = (0..10).map(|_| fork2.rng().next_u64()).collect();

    // All sequences should be different
    assert_ne!(orig_vals, fork1_vals);
    assert_ne!(orig_vals, fork2_vals);
    assert_ne!(fork1_vals, fork2_vals);
}

#[test]
fn test_fork_produces_diverse_hashes() {
    let mut original = PointCloudSampler::with_seed(12345);
    let mut fork1 = original.fork();
    let mut fork2 = original.fork();
    let mut fork3 = original.fork();

    // Generate multiple random integers from each sampler
    let generate_sample = |sampler: &mut PointCloudSampler| -> Vec<u64> {
        (0..20).map(|_| sampler.rng().next_u64()).collect()
    };

    let orig_sample = generate_sample(&mut original);
    let fork1_sample = generate_sample(&mut fork1);
    let fork2_sample = generate_sample(&mut fork2);
    let fork3_sample = generate_sample(&mut fork3);

    // Put all values in a HashSet to check for collisions
    let mut all_values = HashSet::new();
    let mut total_values = 0;

    for sample in [&orig_sample, &fork1_sample, &fork2_sample, &fork3_sample] {
        for &val in sample {
            all_values.insert(val);
            total_values += 1;
        }
    }

    // We should have very few collisions (ideally none, but random is random)
    // With 80 random u64 values, collision probability is extremely low
    let collision_rate = 1.0 - (all_values.len() as f64) / (total_values as f64);

    // Assert that collision rate is very low (less than 5%)
    assert!(collision_rate < 0.05, "Too many hash collisions: {:.2}%", collision_rate * 100.0);

    // Also verify that each fork produced different sequences
    assert_ne!(orig_sample, fork1_sample);
    assert_ne!(orig_sample, fork2_sample);
    assert_ne!(fork1_sample, fork2_sample);
}

#[test]
fn test_fork_deterministic_from_same_seed() {
    // Create two original samplers with the same seed
    let mut original1 = PointCloudSampler::with_seed(999);
    let mut original2 = PointCloudSampler::with_seed(999);

    // Fork each one
    let mut fork1_from_orig1 = original1.fork();
    let mut fork1_from_orig2 = original2.fork();

    // The first forks should produce the same sequence
    let fork1_vals: Vec<u64> = (0..5).map(|_| fork1_from_orig1.rng().next_u64()).collect();
    let fork2_vals: Vec<u64> = (0..5).map(|_| fork1_from_orig2.rng().next_u64()).collect();

    assert_eq!(fork1_vals, fork2_vals);
}

#[test]
fn test_multiple_forks_from_same_original() {
    let mut original = PointCloudSampler::with_seed(777);

    // Create multiple forks
    let mut forks: Vec<PointCloudSampler> = (0..5).map(|_| original.fork()).collect();

    // Generate samples from each fork
    let samples: Vec<Vec<f64>> = forks
        .iter_mut()
        .map(|fork| (0..10).map(|_| fork.rng().next_u64() as f64).collect())
        .collect();

    // Verify all samples are different from each other
    for i in 0..samples.len() {
        for j in i + 1..samples.len() {
            assert_ne!(samples[i], samples[j], "Fork {} and {} produced identical sequences", i, j);
        }
    }
}

#[test]
fn test_original_sampler_advances_after_fork() {
    let mut original = PointCloudSampler::with_seed(555);

    // Get a value before forking
    let val_before_fork: u64 = original.rng().next_u64();

    // Fork the sampler
    let _fork = original.fork();

    // Get a value after forking
    let val_after_fork: u64 = original.rng().next_u64();

    // The original sampler should have advanced (different values)
    assert_ne!(val_before_fork, val_after_fork);
}
