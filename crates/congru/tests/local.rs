#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

use congru::{Distance, Fraction, Icp, Kernel, Metric, RigidTransform, Target};
use congru_synth::{Corruption, MotionBounds, Outliers, Scenario, shapes};

const NOISE: f64 = 0.005;
const NEAR: MotionBounds = MotionBounds {
    max_angle: 0.3,
    max_translation: 0.1,
};
const SEEDS: u64 = 5;

fn errors(icp: &Icp, scenario: &Scenario, target: &Target) -> (f64, f64) {
    let alignment = icp
        .align(&scenario.observed, target, &RigidTransform::identity())
        .unwrap();
    let expected = scenario.expected();
    (
        alignment.transform.rotation_error_to(&expected),
        alignment.transform.translation_error_to(&expected),
    )
}

fn assert_recovers(icp: &Icp, corruption: &Corruption) {
    let model = shapes::wavy_patch(40).unwrap();
    let target = Target::with_estimated_normals(model.clone(), 10).unwrap();
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(&model, corruption, &NEAR, seed).unwrap();
        let (rotation, translation) = errors(icp, &scenario, &target);
        assert!(
            rotation < 10.0 * NOISE,
            "seed {seed}: rotation off by {rotation}"
        );
        assert!(
            translation < 10.0 * NOISE,
            "seed {seed}: translation off by {translation}"
        );
    }
}

fn noisy() -> Corruption {
    Corruption {
        noise: NOISE,
        ..Corruption::default()
    }
}

#[test]
fn point_to_plane_recovers_a_nearby_pose() {
    assert_recovers(&Icp::default(), &noisy());
}

#[test]
fn point_to_point_recovers_a_nearby_pose() {
    let icp = Icp {
        metric: Metric::PointToPoint,
        convergence: congru::Convergence {
            max_iterations: 300,
            ..congru::Convergence::default()
        },
        ..Icp::default()
    };
    assert_recovers(&icp, &noisy());
}

#[test]
fn a_robust_kernel_shrugs_off_outliers() {
    let corruption = Corruption {
        outliers: Outliers::InBox { count: 400 },
        ..noisy()
    };
    let icp = Icp {
        kernel: Kernel::Tukey(Distance::new(10.0 * NOISE).unwrap()),
        ..Icp::default()
    };
    assert_recovers(&icp, &corruption);
}

#[test]
fn trimming_handles_a_partial_view() {
    let corruption = Corruption {
        visible: Fraction::new(0.6).unwrap(),
        outliers: Outliers::InBox { count: 100 },
        ..noisy()
    };
    let icp = Icp {
        overlap: Fraction::new(0.85).unwrap(),
        kernel: Kernel::Cauchy(Distance::new(3.0 * NOISE).unwrap()),
        ..Icp::default()
    };
    assert_recovers(&icp, &corruption);
}

#[test]
fn point_to_plane_needs_normals() {
    let model = shapes::wavy_patch(10).unwrap();
    let target = Target::new(model.clone()).unwrap();
    let result = Icp::default().align(&model, &target, &RigidTransform::identity());
    assert_eq!(result, Err(congru::Error::MissingNormals));
}
