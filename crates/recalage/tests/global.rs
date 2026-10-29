use recalage::{Distance, Fraction, Icp, Kernel, Multistart, PointCloud, RigidTransform, Target};
use recalage_synth::{Corruption, MotionBounds, Scenario, shapes};

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
fn a_dozen_marks_align_onto_a_mesh_from_a_rough_pose() {
    let mesh = shapes::lumpy_sphere_mesh(64).unwrap();
    let target = Target::from_mesh(&mesh, Distance::new(0.01).unwrap()).unwrap();
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
    let icp = Icp {
        kernel: Kernel::Huber(Distance::new(NOISE).unwrap()),
        ..Icp::default()
    };
    for seed in 0..SEEDS {
        let scenario = Scenario::generate(&surface, &corruption, &rough, seed).unwrap();
        assert!(scenario.observed.count() >= 10);
        let alignment = icp
            .align(&scenario.observed, &target, &RigidTransform::identity())
            .unwrap();
        let landed: PointCloud = scenario.observed.transformed(&alignment.transform);
        let residual = landed
            .points()
            .iter()
            .map(|point| target.index().nearest(point).squared_distance.sqrt())
            .fold(0.0, f64::max);
        assert!(
            residual < 4.0 * NOISE,
            "seed {seed}: a mark lands {residual} off the surface"
        );
    }
}
