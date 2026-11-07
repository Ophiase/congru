use std::f64::consts::PI;

use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use super::shapes::{clover, marks, sheet, truth};
use super::*;
use crate::geometry::count_as_float;

pub(super) const TRUNCATION: f64 = 0.02;
pub(super) const REACH: f64 = 0.15;
pub(super) const RESOLUTION: [f64; 2] = [0.04, 0.004];

pub(super) fn certifier(margin: f64, budget: usize) -> Result<Certifier, Error> {
    Ok(Certifier {
        truncation: Distance::new(TRUNCATION)?,
        reach: Distance::new(REACH)?,
        margin,
        resolution: RESOLUTION,
        budget,
    })
}

pub(super) fn value_of(
    source: &PointCloud,
    target: &PointCloud,
    pose: &RigidTransform,
) -> Result<f64, Error> {
    let index = NearestNeighbors::new(target)?;
    let total: f64 = source
        .points()
        .iter()
        .map(|point| {
            index
                .nearest(&pose.apply(point))
                .squared_distance
                .min(TRUNCATION * TRUNCATION)
        })
        .sum();
    Ok(total / count_as_float(source.count()))
}

pub(super) fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

pub(super) fn random_pose(
    random: &mut ChaCha8Rng,
    source: &PointCloud,
    target: &PointCloud,
) -> RigidTransform {
    let turned = nalgebra::UnitQuaternion::from_scaled_axis(drawn(random, PI));
    RigidTransform::from_parts(
        turned,
        target.centroid().coords + drawn(random, REACH) - turned * source.centroid().coords,
    )
}

/// How far the survivor farthest from `pose` is turned from it, and the nearest.
pub(super) fn spread(certificate: &Certificate, pose: &RigidTransform) -> (f64, f64) {
    certificate.survivors.iter().fold(
        (0.0, f64::INFINITY),
        |(farthest, nearest): (f64, f64), survivor| {
            let angle = survivor.rotation_error_to(pose);
            (farthest.max(angle), nearest.min(angle))
        },
    )
}

#[test]
fn the_truth_always_survives() -> Result<(), Error> {
    let target = sheet()?;
    let source = marks(&target, 40)?;
    let certificate = certifier(0.0, 3_000_000)?.run(&source, &target, &[])?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    let (_, nearest) = spread(&certificate, &truth());
    assert!(nearest < 2.0 * RESOLUTION[0], "nearest {nearest} rad");
    Ok(())
}

#[test]
fn more_marks_exclude_more_poses() -> Result<(), Error> {
    let target = sheet()?;
    let run = certifier(0.0, 3_000_000)?;
    let few = run.run(&marks(&target, 6)?, &target, &[])?;
    let many = run.run(&marks(&target, 60)?, &target, &[])?;
    assert!(many.closed, "open after {}", many.measured);
    let (far_few, _) = spread(&few, &truth());
    let (far_many, _) = spread(&many, &truth());
    assert!(far_few > 0.5, "six marks keep only {far_few} rad");
    assert!(far_many < 0.2, "sixty marks still keep {far_many} rad");
    Ok(())
}

#[test]
fn no_drawn_pose_ever_passes_the_bound() -> Result<(), Error> {
    let target = sheet()?;
    let source = marks(&target, 12)?;
    let certificate = certifier(0.0, 30_000)?.run(&source, &target, &[])?;
    assert!(value_of(&source, &target, &truth())? >= certificate.lower);
    let mut random = ChaCha8Rng::seed_from_u64(5);
    for _ in 0..20_000 {
        let pose = random_pose(&mut random, &source, &target);
        assert!(value_of(&source, &target, &pose)? >= certificate.lower);
    }
    Ok(())
}

#[test]
fn every_pose_inside_a_cube_stays_above_its_bound() -> Result<(), Error> {
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
    let mut random = ChaCha8Rng::seed_from_u64(9);
    for half_rotation in [0.02, 0.2, 1.0, PI] {
        for half_translation in [0.001, 0.01, 0.1] {
            let cube = problem.measured(
                Cube {
                    rotation: Vector::new(0.4, -0.3, 0.2),
                    translation: target.centroid().coords,
                    half_rotation,
                    half_translation,
                    lower: 0.0,
                    value: 0.0,
                },
                f64::INFINITY,
            );
            for _ in 0..300 {
                let inside = Cube {
                    rotation: cube.rotation + drawn(&mut random, half_rotation),
                    translation: cube.translation + drawn(&mut random, half_translation),
                    ..cube
                };
                let value = problem.value(&problem.pose(&inside));
                assert!(value >= cube.lower, "{value} under {}", cube.lower);
            }
        }
    }
    Ok(())
}

#[test]
fn a_wrong_seed_is_shown_to_be_far() -> Result<(), Error> {
    let target = sheet()?;
    let source = marks(&target, 12)?;
    let wrong = RigidTransform::from_axis_angle(Vector::new(0.0, 2.5, 0.0), [0.0; 3]);
    let certificate = certifier(-1e-5, 3_000_000)?.run(&source, &target, &[wrong])?;
    let value = value_of(&source, &target, &wrong)?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    assert!(certificate.gap(value) > 1e-4, "{}", certificate.gap(value));
    Ok(())
}

#[test]
fn a_clover_keeps_the_turns_its_symmetry_allows() -> Result<(), Error> {
    let target = clover()?;
    let source = marks(&target, 40)?;
    let certificate = certifier(0.0, 3_000_000)?.run(&source, &target, &[])?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    let turn = RigidTransform::from_axis_angle(Vector::new(0.0, 0.0, PI / 2.0), [0.0; 3]);
    let (_, nearest) = spread(&certificate, &truth().then(&turn));
    assert!(nearest < 2.0 * RESOLUTION[0], "nearest {nearest} rad");
    Ok(())
}

#[test]
fn a_resolution_of_nothing_is_refused() -> Result<(), Error> {
    let target = sheet()?;
    let run = Certifier {
        resolution: [0.0, 0.001],
        ..certifier(0.0, 10)?
    };
    assert!(run.run(&target, &target, &[]).is_err());
    Ok(())
}
