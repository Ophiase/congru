use std::f64::consts::PI;

use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use super::shapes::{marks, sheet, truth};
use super::tests::{
    REACH, RESOLUTION, TRUNCATION, certifier, drawn, random_pose, spread, value_of,
};
use super::*;

#[test]
fn a_stopped_sum_keeps_every_cube_the_full_sum_keeps() -> Result<(), Error> {
    let target = sheet()?;
    let source = marks(&target, 12)?;
    let index = NearestNeighbors::new(&target)?;
    let problem = problem_of(
        &source,
        Shape::Points {
            index: &index,
            cloud: &target,
        },
        TRUNCATION,
    );
    let mut random = ChaCha8Rng::seed_from_u64(13);
    let tau_squared = TRUNCATION * TRUNCATION;
    for _ in 0..3000 {
        let cube = Cube {
            rotation: drawn(&mut random, PI),
            translation: target.centroid().coords + drawn(&mut random, REACH),
            half_rotation: 0.1 * random.random::<f64>(),
            half_translation: 0.01 * random.random::<f64>(),
            lower: 0.0,
            value: 0.0,
        };
        let full = problem.measured(cube, f64::INFINITY);
        let ceiling = tau_squared * random.random::<f64>();
        let stopped = problem.measured(cube, ceiling);
        assert!(stopped.lower <= full.lower + 1e-15);
        if full.lower <= ceiling {
            assert!((stopped.lower - full.lower).abs() < 1e-15);
            assert!((stopped.value - full.value).abs() < 1e-15);
        } else if stopped.value.is_infinite() {
            assert!(stopped.lower > ceiling * (1.0 - 1e-12));
        }
    }
    Ok(())
}

#[test]
fn a_half_turn_stays_in_the_search() -> Result<(), Error> {
    let target = sheet()?;
    let on_x = Vector::new(PI, 0.0, 0.0);
    let on_diagonal = Vector::new(1.0, 1.0, 1.0).normalize() * PI;
    for turn in [on_x, -on_x, on_diagonal] {
        let pose = RigidTransform::from_axis_angle(turn, [0.03, -0.02, 0.01]);
        let moved: Vec<Point> = target
            .points()
            .iter()
            .step_by(15)
            .map(|point| pose.inverse().apply(point))
            .collect();
        let source = PointCloud::new(moved)?;
        let certificate = certifier(0.0, 3_000_000)?.run(&source, &target, &[])?;
        assert!(certificate.closed, "open after {}", certificate.measured);
        let (_, nearest) = spread(&certificate, &pose);
        assert!(nearest < 2.0 * RESOLUTION[0], "nearest {nearest} rad");
    }
    Ok(())
}

#[test]
fn no_pose_near_a_half_turn_passes_the_bound() -> Result<(), Error> {
    let target = sheet()?;
    let source = marks(&target, 12)?;
    let certificate = certifier(0.0, 30_000)?.run(&source, &target, &[])?;
    let mut random = ChaCha8Rng::seed_from_u64(17);
    for _ in 0..20_000 {
        let angle = PI - 0.02 * random.random::<f64>();
        let turn = drawn(&mut random, 1.0).normalize() * angle;
        let pose = RigidTransform::from_axis_angle(turn, [0.0; 3]);
        let shifted = RigidTransform::from_parts(
            pose.rotation(),
            target.centroid().coords + drawn(&mut random, REACH)
                - pose.rotation() * source.centroid().coords,
        );
        assert!(value_of(&source, &target, &shifted)? >= certificate.lower);
    }
    Ok(())
}

#[test]
fn strays_close_and_no_drawn_pose_passes_the_bound() -> Result<(), Error> {
    let target = sheet()?;
    let mut random = ChaCha8Rng::seed_from_u64(21);
    let mut points = marks(&target, 12)?.points().to_vec();
    for _ in 0..3 {
        let stray = target.centroid() + drawn(&mut random, 0.1);
        points.push(truth().inverse().apply(&stray));
    }
    let source = PointCloud::new(points)?;
    let at_truth = value_of(&source, &target, &truth())?;
    let floor = (at_truth.sqrt() - 0.001).max(0.0);
    let certificate =
        certifier(floor * floor - at_truth, 15_000_000)?.run(&source, &target, &[])?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    assert!(certificate.lower <= at_truth);
    let at_pose = value_of(&source, &target, &certificate.pose)?;
    assert!((at_pose - certificate.upper).abs() < 1e-12);
    for _ in 0..20_000 {
        let pose = random_pose(&mut random, &source, &target);
        assert!(value_of(&source, &target, &pose)? >= certificate.lower);
    }
    Ok(())
}
