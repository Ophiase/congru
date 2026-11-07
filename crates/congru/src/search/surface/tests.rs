use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};

use super::triangle::Face;
use super::*;
use crate::{Distance, Error, Icp, PointCloud, RigidTransform, TriangleMesh};

const QUERIES: usize = 2_000;
const SIDE: f64 = 0.2;

/// A box of the given sides, its faces turned outwards.
fn box_mesh(sides: [f64; 3]) -> Result<TriangleMesh, Error> {
    let corners: Vec<Point> = (0..8)
        .map(|bits| {
            Point::new(
                sides[0] * f64::from(bits & 1),
                sides[1] * f64::from(bits >> 1 & 1),
                sides[2] * f64::from(bits >> 2 & 1),
            )
        })
        .collect();
    let triangles = [
        [0, 2, 1],
        [1, 2, 3],
        [4, 5, 6],
        [5, 7, 6],
        [0, 1, 4],
        [1, 5, 4],
        [2, 6, 3],
        [3, 6, 7],
        [0, 4, 2],
        [2, 4, 6],
        [1, 3, 5],
        [3, 7, 5],
    ];
    TriangleMesh::new(PointCloud::new(corners)?, triangles.to_vec())
}

fn drawn(random: &mut ChaCha8Rng, low: f64, high: f64) -> Point {
    Point::from(Vector::from_fn(|_, _| {
        low + (high - low) * random.random::<f64>()
    }))
}

/// The distance to the box, known in closed form: the truth of what follows.
fn truth(query: &Point, sides: [f64; 3]) -> f64 {
    let outside = Vector::from_fn(|axis, _| (query[axis] - sides[axis]).max(-query[axis]).max(0.0));
    if outside.norm() > 0.0 {
        return outside.norm();
    }
    (0..3)
        .map(|axis| query[axis].min(sides[axis] - query[axis]))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn the_tree_gives_the_exact_distance_to_a_box() -> Result<(), Error> {
    let sides = [SIDE, 0.13, 0.07];
    let index = MeshIndex::new(&box_mesh(sides)?)?;
    let mut random = ChaCha8Rng::seed_from_u64(1);
    for _ in 0..QUERIES {
        let query = drawn(&mut random, -0.1, 0.3);
        let found = index.closest(&query).ok_or(Error::Degenerate)?;
        let error = (found.squared_distance.sqrt() - truth(&query, sides)).abs();
        assert!(error < 1e-12, "{error} at {query:?}");
    }
    Ok(())
}

#[test]
fn the_tree_agrees_with_every_face_of_a_large_mesh() -> Result<(), Error> {
    let mut random = ChaCha8Rng::seed_from_u64(2);
    let corners: Vec<Point> = (0..600).map(|_| drawn(&mut random, 0.0, 1.0)).collect();
    let triangles: Vec<[u32; 3]> = (0..200_u32)
        .map(|at| [3 * at, 3 * at + 1, 3 * at + 2])
        .collect();
    let mesh = TriangleMesh::new(PointCloud::new(corners.clone())?, triangles)?;
    let index = MeshIndex::new(&mesh)?;
    let faces: Vec<Face> = corners
        .chunks_exact(3)
        .filter_map(|three| Face::new([three[0], three[1], three[2]]))
        .collect();
    for _ in 0..QUERIES {
        let query = drawn(&mut random, -0.5, 1.5);
        let every = faces
            .iter()
            .map(|face| (face.closest(&query) - query).norm_squared())
            .fold(f64::INFINITY, f64::min);
        let found = index.closest(&query).ok_or(Error::Degenerate)?;
        assert!((found.squared_distance - every).abs() < 1e-12);
    }
    Ok(())
}

#[test]
fn the_field_is_within_a_voxel_of_the_truth_with_its_sign() -> Result<(), Error> {
    const VOXEL: f64 = 0.002;
    const BAND: f64 = 0.02;
    let sides = [SIDE, 0.13, 0.07];
    let field = DistanceField::from_mesh(
        &box_mesh(sides)?,
        Distance::new(VOXEL)?,
        Distance::new(BAND)?,
    )?;
    let mut random = ChaCha8Rng::seed_from_u64(3);
    let mut checked = 0;
    for _ in 0..QUERIES {
        let query = drawn(&mut random, -0.01, 0.21);
        let inside = (0..3).all(|axis| (0.0..sides[axis]).contains(&query[axis]));
        let exact = truth(&query, sides);
        if exact > BAND - 2.0 * VOXEL {
            continue;
        }
        let (value, _) = field.read(&query).ok_or(Error::Degenerate)?;
        let expected = if inside { -exact } else { exact };
        assert!(
            (value - expected).abs() < 2.0 * VOXEL,
            "{value} for {expected}"
        );
        checked += 1;
    }
    assert!(checked > QUERIES / 10, "{checked} queries in the band");
    Ok(())
}

#[test]
fn the_field_points_back_at_a_flat_face() -> Result<(), Error> {
    const VOXEL: f64 = 0.002;
    let sides = [SIDE, SIDE, SIDE];
    let field = DistanceField::from_mesh(
        &box_mesh(sides)?,
        Distance::new(VOXEL)?,
        Distance::new(0.02)?,
    )?;
    let query = Point::new(0.1, 0.1, 0.207);
    let found = field.closest(&query).ok_or(Error::Degenerate)?;
    assert!((found.point - Point::new(0.1, 0.1, 0.2)).norm() < VOXEL);
    assert!(found.normal.z > 0.99, "{:?}", found.normal);
    Ok(())
}

#[test]
fn a_motion_is_recovered_on_the_mesh_and_on_its_field() -> Result<(), Error> {
    const NOISE: f64 = 0.0005;
    let mesh = box_mesh([SIDE, 0.13, 0.07])?;
    let truth =
        RigidTransform::from_axis_angle(Vector::new(0.03, -0.02, 0.04), [0.004, 0.0, 0.002]);
    let mut random = ChaCha8Rng::seed_from_u64(4);
    let moved: Vec<Point> = mesh
        .sample_surface(Distance::new(0.006)?)?
        .cloud()
        .points()
        .iter()
        .map(|point| {
            truth.inverse().apply(point) + Vector::from_fn(|_, _| NOISE * random.random::<f64>())
        })
        .collect();
    let source = PointCloud::new(moved)?;
    let index = MeshIndex::new(&mesh)?;
    let field = DistanceField::from_mesh(&mesh, Distance::new(0.002)?, Distance::new(0.03)?)?;
    let start = RigidTransform::identity();
    let icp = Icp::default();
    for found in [
        icp.align_on(&source, &index, &start)?,
        icp.align_on(&source, &field, &start)?,
    ] {
        assert!(found.transform.rotation_error_to(&truth) < 0.01);
        assert!((found.transform.translation() - truth.translation()).norm() < 0.002);
    }
    Ok(())
}

#[test]
fn past_the_band_a_field_says_nothing() -> Result<(), Error> {
    let field = DistanceField::from_mesh(
        &box_mesh([SIDE; 3])?,
        Distance::new(0.002)?,
        Distance::new(0.01)?,
    )?;
    assert!(field.closest(&Point::new(0.1, 0.1, 0.205)).is_some());
    assert!(field.closest(&Point::new(0.1, 0.1, 0.5)).is_none());
    assert!(field.closest(&Point::new(0.1, 0.1, 0.25)).is_none());
    Ok(())
}
