use congru::{Distance, Fraction, Multistart, Target, fit_rigid};
use congru_synth::{Corruption, MotionBounds, Scenario, shapes};

const NOISE: f64 = 0.01;
const SEEDS: u64 = 4;
const TOLERANCE: f64 = 0.05;

#[test]
fn multistart_recovers_any_rotation() {
    let model = shapes::lumpy_sphere(2000).unwrap();
    let target = Target::with_estimated_normals(model.clone(), 12).unwrap();
    let corruption = Corruption {
        noise: NOISE,
        ..Corruption::default()
    };
    for seed in 0..SEEDS {
        let scenario =
            Scenario::generate(&model, &corruption, &MotionBounds::any_rotation(1.0), seed)
                .unwrap();
        let alignment = Multistart::default()
            .align(&scenario.observed, &target)
            .unwrap();
        let error = alignment.transform.rotation_error_to(&scenario.expected());
        assert!(error < TOLERANCE, "seed {seed}: rotation off by {error}");
    }
}

#[test]
fn a_mesh_target_aligns_a_scan() {
    let mesh = shapes::lumpy_sphere_mesh(48).unwrap();
    let target = Target::from_mesh(&mesh, Distance::new(0.02).unwrap()).unwrap();
    let scan = shapes::lumpy_sphere(3000).unwrap();
    let corruption = Corruption {
        noise: NOISE,
        ..Corruption::default()
    };
    let scenario =
        Scenario::generate(&scan, &corruption, &MotionBounds::any_rotation(1.0), 1).unwrap();
    let alignment = Multistart::default()
        .align(&scenario.observed, &target)
        .unwrap();
    assert!(alignment.transform.rotation_error_to(&scenario.expected()) < TOLERANCE);
}

#[test]
fn numbered_marks_give_the_pose_that_icp_cannot() {
    let surface = shapes::lumpy_sphere(400).unwrap();
    let corruption = Corruption {
        noise: NOISE / 2.0,
        visible: Fraction::new(0.03).unwrap(),
        ..Corruption::default()
    };
    let rough = MotionBounds {
        max_angle: 0.25,
        max_translation: 0.05,
    };
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(&surface, &corruption, &rough, seed).unwrap();
        let truth = scenario.observed.transformed(&scenario.expected());
        let fit = fit_rigid(scenario.observed.points(), truth.points(), None).unwrap();
        assert!(fit.transform.rotation_error_to(&scenario.expected()) < 1e-9);
    }
}
