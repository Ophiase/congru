use std::f64::consts::{FRAC_PI_2, PI};

use nalgebra::UnitQuaternion;
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use super::*;
use crate::{Surface, TriangleMesh};

fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
    Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
}

/// The lemma the bound stands on: two rotation vectors `r` and `q` turn a point `x`
/// at most `2 sin(min(|r - q| / 2, pi / 2)) |x|` apart, wherever they lie.
#[test]
fn two_rotation_vectors_move_a_point_no_further_than_the_lemma_says() {
    let mut random = ChaCha8Rng::seed_from_u64(21);
    for draw in 0..1_000_000_u32 {
        let spread = [1e-3, 0.1, 1.0, PI][(draw % 4) as usize];
        let first = drawn(&mut random, PI);
        let second = first + drawn(&mut random, spread);
        let point = drawn(&mut random, 1.0);
        let apart = (UnitQuaternion::from_scaled_axis(first) * point
            - UnitQuaternion::from_scaled_axis(second) * point)
            .norm();
        let angle = ((first - second).norm() / 2.0).min(FRAC_PI_2);
        let bound = 2.0 * angle.sin() * point.norm();
        assert!(apart <= bound + 1e-12, "{apart} past {bound}");
    }
}

/// A plate with a wedge on one corner, so that no turn leaves it as it was.
fn wedge() -> Result<TriangleMesh, Error> {
    let points = [
        [0.0, 0.0, 0.0],
        [0.2, 0.0, 0.0],
        [0.2, 0.13, 0.0],
        [0.0, 0.13, 0.0],
        [0.05, 0.03, 0.09],
    ]
    .map(Point::from);
    let triangles = [
        [0_u32, 2, 1],
        [0, 3, 2],
        [0, 1, 4],
        [1, 2, 4],
        [2, 3, 4],
        [3, 0, 4],
    ];
    TriangleMesh::new(PointCloud::new(points.to_vec())?, triangles.to_vec())
}

fn run_on(
    margin: f64,
    budget: usize,
) -> Result<(Certificate, PointCloud, RigidTransform, MeshIndex), Error> {
    let mesh = wedge()?;
    let index = MeshIndex::new(&mesh)?;
    let truth = RigidTransform::from_axis_angle(Vector::new(0.9, -0.5, 0.4), [0.02, -0.01, 0.03]);
    let moved: Vec<Point> = mesh
        .sample_surface(Distance::new(0.03)?)?
        .cloud()
        .points()
        .iter()
        .map(|point| truth.inverse().apply(point))
        .collect();
    let source = PointCloud::new(moved)?;
    let certificate = Certifier {
        truncation: Distance::new(0.02)?,
        reach: Distance::new(0.1)?,
        margin,
        resolution: [0.02, 0.002],
        budget,
    }
    .run_on_mesh(&source, &index, &[])?;
    Ok((certificate, source, truth, index))
}

fn value_on(source: &PointCloud, mesh: &MeshIndex, pose: &RigidTransform) -> f64 {
    let total: f64 = source
        .points()
        .iter()
        .map(|point| {
            mesh.closest(&pose.apply(point))
                .map_or(f64::INFINITY, |closest| closest.squared_distance)
                .min(0.02 * 0.02)
        })
        .sum();
    total / crate::geometry::count_as_float(source.count())
}

#[test]
fn on_a_mesh_the_truth_survives_and_no_pose_passes_the_bound() -> Result<(), Error> {
    let (certificate, source, truth, index) = run_on(0.0, 3_000_000)?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    assert!(value_on(&source, &index, &truth) >= certificate.lower);
    let near = certificate
        .survivors
        .iter()
        .map(|pose| pose.rotation_error_to(&truth))
        .fold(f64::INFINITY, f64::min);
    assert!(near < 0.1, "nearest survivor {near} rad off");
    let mut random = ChaCha8Rng::seed_from_u64(22);
    for _ in 0..5_000 {
        let turned = UnitQuaternion::from_scaled_axis(drawn(&mut random, PI));
        let pose = RigidTransform::from_parts(
            turned,
            index.centre() + drawn(&mut random, 0.1) - turned * source.centroid().coords,
        );
        assert!(value_on(&source, &index, &pose) >= certificate.lower);
    }
    Ok(())
}

#[test]
fn on_a_mesh_the_best_value_is_that_of_its_pose() -> Result<(), Error> {
    let (certificate, source, _, index) = run_on(-1e-5, 3_000_000)?;
    assert!(certificate.closed, "open after {}", certificate.measured);
    let value = value_on(&source, &index, &certificate.pose);
    assert!((value - certificate.upper).abs() < 1e-12);
    assert!(certificate.upper < 1e-5, "{}", certificate.upper);
    Ok(())
}
