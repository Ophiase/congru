#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

use congru::{Distance, Fraction, Icp, MotionKind, PointCloud, RigidTransform, Target};
use congru_synth::{Corruption, MotionBounds, Scenario, shapes};

const SEEDS: u64 = 4;
const NOISE: f64 = 0.005;
const FAR_AXIS: f64 = 1.5;

fn mesh_target() -> Target {
    Target::from_mesh(
        &shapes::lumpy_sphere_mesh(64).unwrap(),
        Distance::new(0.01).unwrap(),
    )
    .unwrap()
}

fn observed(surface: &PointCloud, visible: f64, seed: u64) -> PointCloud {
    let corruption = Corruption {
        noise: NOISE,
        visible: Fraction::new(visible).unwrap(),
        ..Corruption::default()
    };
    let still = MotionBounds {
        max_angle: 0.0,
        max_translation: 0.0,
    };
    Scenario::generate(surface, &corruption, &still, seed)
        .unwrap()
        .observed
}

#[test]
fn marks_on_a_rounded_patch_report_a_slide_along_the_surface() {
    let target = mesh_target();
    let surface = shapes::lumpy_sphere(400).unwrap();
    for seed in 0..SEEDS {
        let marks = observed(&surface, 0.03, seed);
        let alignment = Icp::default()
            .align(&marks, &target, &RigidTransform::identity())
            .unwrap();
        let landed = marks.transformed(&alignment.transform);
        let centre = landed.centroid();
        let spread = landed
            .points()
            .iter()
            .map(|point| (point - centre).norm())
            .fold(0.0, f64::max);
        let report = alignment.observability.unwrap();
        let slides = report.weak_motions.iter().any(|motion| match motion.kind {
            MotionKind::Rotation { axis, through } => {
                (centre - through).cross(&axis).norm() > FAR_AXIS * spread
            }
            MotionKind::Translation { .. } => true,
        });
        assert!(slides, "seed {seed}: {report:?}");
    }
}

#[test]
fn a_whole_scan_pins_every_motion() {
    let target = mesh_target();
    let scan = observed(&shapes::lumpy_sphere(3000).unwrap(), 1.0, 0);
    let alignment = Icp::default()
        .align(&scan, &target, &RigidTransform::identity())
        .unwrap();
    let report = alignment.observability.unwrap();
    assert!(report.weak_motions.is_empty(), "{report:?}");
}
