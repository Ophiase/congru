use recalage::{Error, Point, RigidTransform, fit_rigid};
use recalage_synth::{MotionBounds, shapes};

const SEEDS: u64 = 8;
const EXACT: f64 = 1e-9;

fn moved(points: &[Point], motion: &RigidTransform) -> Vec<Point> {
    points.iter().map(|point| motion.apply(point)).collect()
}

#[test]
fn recovers_any_motion_from_exact_pairs() {
    let model = shapes::lumpy_sphere(50).unwrap();
    for seed in 0..SEEDS {
        let mut random = <rand::rngs::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
        let truth = MotionBounds::any_rotation(10.0).draw(&mut random);
        let fit = fit_rigid(model.points(), &moved(model.points(), &truth), None).unwrap();
        assert!(fit.transform.rotation_error_to(&truth) < EXACT);
        assert!(fit.transform.translation_error_to(&truth) < EXACT);
        assert!(fit.rmse < EXACT);
    }
}

#[test]
fn a_zero_weight_silences_a_wrong_pair() {
    let model = shapes::lumpy_sphere(20).unwrap();
    let truth = RigidTransform::from_axis_angle([0.3, -0.2, 0.9].into(), [1.0, 2.0, 3.0]);
    let mut target = moved(model.points(), &truth);
    target[0] += nalgebra::Vector3::repeat(100.0);
    let mut weights = vec![1.0; target.len()];
    weights[0] = 0.0;
    let fit = fit_rigid(model.points(), &target, Some(&weights)).unwrap();
    assert!(fit.transform.rotation_error_to(&truth) < EXACT);
}

#[test]
fn collinear_points_leave_the_motion_undetermined() {
    let line: Vec<Point> = (0..5)
        .map(|step| Point::new(f64::from(step), 0.0, 0.0))
        .collect();
    assert_eq!(fit_rigid(&line, &line, None), Err(Error::Degenerate));
}

#[test]
fn mismatched_pairs_are_refused() {
    let model = shapes::lumpy_sphere(10).unwrap();
    let result = fit_rigid(model.points(), &model.points()[1..], None);
    assert!(matches!(result, Err(Error::LengthMismatch { .. })));
}
