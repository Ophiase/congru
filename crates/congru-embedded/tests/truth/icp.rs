use congru::PointCloud;
use congru_embedded::{Convergence, Icp, KdTree, Kernel, Metric, Pose, Real, Workspace};
use congru_synth::{Corruption, MotionBounds, Outliers, Scenario, shapes};

use crate::fixtures::{cloud_of, errors, raised, rotation_floor, translation_floor};

const NOISE: f64 = 0.005;
const NEAR: MotionBounds = MotionBounds {
    max_angle: 0.3,
    max_translation: 0.1,
};
/// On 40 × 40 points over the unit square no point moves half a spacing, so
/// every first pair is right; farther away, point-to-point may lock onto
/// wrong pairs, a minimum of its own that no precision would change.
const WITHIN_HALF_A_SPACING: MotionBounds = MotionBounds {
    max_angle: 0.005,
    max_translation: 0.005,
};
const SEEDS: u64 = 5;
const NORMAL_PATCH: usize = 10;
/// The wavy patch lies over the unit square, about its corner.
const EXTENT: f64 = 1.0;

fn target_of<Float: Real>(model: &PointCloud) -> KdTree<Float> {
    let mut tree = KdTree::new(&cloud_of(model)).unwrap();
    tree.estimate_normals::<NORMAL_PATCH>().unwrap();
    tree
}

fn icp_of<Float: Real>(metric: Metric, target: &KdTree<Float>) -> Icp<Float> {
    Icp {
        metric,
        convergence: Convergence {
            max_iterations: 300,
            ..Convergence::for_extent(target.extent())
        },
        ..Icp::default()
    }
}

fn run<Float: Real>(
    icp: &Icp<Float>,
    corruption: &Corruption,
    motion: &MotionBounds,
    bounds: (f64, f64),
) {
    let model = shapes::wavy_patch(40).unwrap();
    let target = target_of::<Float>(&model);
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(&model, corruption, motion, seed).unwrap();
        let source = cloud_of::<Float>(&scenario.observed);
        let mut workspace = Workspace::new(source.len());
        let found = icp
            .align(&source, &target, &Pose::identity(), &mut workspace)
            .unwrap();
        let (rotation, translation) = errors(&found.pose, &scenario.expected());
        assert!(
            rotation < bounds.0,
            "seed {seed}: {rotation} / {}",
            bounds.0
        );
        assert!(
            translation < bounds.1,
            "seed {seed}: {translation} / {}",
            bounds.1
        );
    }
}

fn noisy() -> Corruption {
    Corruption {
        noise: NOISE,
        ..Corruption::default()
    }
}

/// Nothing but rounding and the last step's tolerance separates the clouds.
/// With every pair right, the next solve is exact, so a step below tolerance
/// leaves no more than itself.
fn exact<Float: Real>(metric: Metric, motion: &MotionBounds) {
    let corruption = Corruption::default();
    let model = shapes::wavy_patch(40).unwrap();
    let target = target_of::<Float>(&model);
    let icp = icp_of(metric, &target);
    run(
        &icp,
        &corruption,
        motion,
        (
            rotation_floor::<Float>(EXTENT, EXTENT) + icp.convergence.rotation.as_f64(),
            translation_floor::<Float>(EXTENT, EXTENT) + icp.convergence.translation.as_f64(),
        ),
    );
}

#[test]
fn identical_samples_are_aligned_to_the_rounding_floor() {
    exact::<f32>(Metric::PointToPlane, &NEAR);
    exact::<f64>(Metric::PointToPlane, &NEAR);
    exact::<f32>(Metric::PointToPoint, &WITHIN_HALF_A_SPACING);
    exact::<f64>(Metric::PointToPoint, &WITHIN_HALF_A_SPACING);
}

/// The main crate's own bound on the same scenes, plus what the scalar loses.
fn within_noise<Float: Real>(metric: Metric, corruption: &Corruption, kernel: Kernel<Float>) {
    let model = shapes::wavy_patch(10).unwrap();
    let icp = Icp {
        kernel,
        ..icp_of(metric, &target_of::<Float>(&model))
    };
    run(
        &icp,
        corruption,
        &NEAR,
        (
            10.0 * NOISE + rotation_floor::<Float>(EXTENT, EXTENT),
            10.0 * NOISE + translation_floor::<Float>(EXTENT, EXTENT),
        ),
    );
}

#[test]
fn noise_costs_no_more_than_in_the_main_crate() {
    within_noise::<f32>(Metric::PointToPlane, &noisy(), Kernel::Quadratic);
    within_noise::<f64>(Metric::PointToPlane, &noisy(), Kernel::Quadratic);
    within_noise::<f32>(Metric::PointToPoint, &noisy(), Kernel::Quadratic);
    within_noise::<f64>(Metric::PointToPoint, &noisy(), Kernel::Quadratic);
}

#[test]
fn a_robust_kernel_shrugs_off_outliers() {
    let corruption = Corruption {
        outliers: Outliers::InBox { count: 400 },
        ..noisy()
    };
    let scale = 10.0 * NOISE;
    within_noise::<f32>(
        Metric::PointToPlane,
        &corruption,
        Kernel::Tukey(Real::of(scale)),
    );
    within_noise::<f64>(Metric::PointToPlane, &corruption, Kernel::Tukey(scale));
}

/// A stop at `16 ε` of a step whose error contracts at worst by half per
/// iteration leaves about as much again; noise and neighbours are the same.
#[test]
fn f32_lands_where_f64_lands_within_its_own_precision() {
    let model = shapes::wavy_patch(40).unwrap();
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(&model, &noisy(), &NEAR, seed).unwrap();
        let narrow = raised(&solved::<f32>(&model, &scenario));
        let wide = solved::<f64>(&model, &scenario);
        let rotation = narrow.rotation_error_to(&wide);
        let translation = narrow.translation_error_to(&wide);
        assert!(rotation < rotation_floor::<f32>(EXTENT, EXTENT));
        assert!(translation < translation_floor::<f32>(EXTENT, EXTENT));
    }
}

fn solved<Float: Real>(model: &PointCloud, scenario: &Scenario) -> Pose<Float> {
    let target = target_of::<Float>(model);
    let source = cloud_of::<Float>(&scenario.observed);
    icp_of(Metric::PointToPlane, &target)
        .align(
            &source,
            &target,
            &Pose::identity(),
            &mut Workspace::new(source.len()),
        )
        .unwrap()
        .pose
}

#[test]
fn a_used_workspace_changes_nothing() {
    let model = shapes::wavy_patch(20).unwrap();
    let target = target_of::<f32>(&model);
    let scenario = Scenario::generate(&model, &noisy(), &NEAR, 1).unwrap();
    let source = cloud_of::<f32>(&scenario.observed);
    let icp = icp_of(Metric::PointToPlane, &target);
    let mut workspace = Workspace::new(source.len());
    let first = icp.align(&source, &target, &Pose::identity(), &mut workspace);
    let second = icp.align(&source, &target, &Pose::identity(), &mut workspace);
    assert_eq!(first, second);
}
