//! Deterministic test shapes, chosen without symmetry so that one rigid
//! motion, and only one, aligns them.

use std::f64::consts::{PI, TAU};

use nalgebra::Vector3;
use congru::{Error, Point, PointCloud, TriangleMesh};

use crate::casts::as_float;

const GOLDEN_ANGLE: f64 = PI * 0.763_932_022_500_210_3;

/// A closed lumpy surface about the unit sphere, `count` points spread
/// evenly by a Fibonacci lattice.
///
/// # Errors
/// [`Error::Empty`] for zero points.
pub fn lumpy_sphere(count: usize) -> Result<PointCloud, Error> {
    let total = as_float(count);
    PointCloud::new(
        (0..count)
            .map(|index| {
                let rank = as_float(index) + 0.5;
                let height = 1.0 - 2.0 * rank / total;
                let ring = (1.0 - height * height).sqrt();
                let angle = GOLDEN_ANGLE * rank;
                lumpy(Vector3::new(ring * angle.cos(), ring * angle.sin(), height))
            })
            .collect::<Vec<_>>(),
    )
}

/// The same lumpy surface as a closed triangle mesh, `rings` latitudes by
/// `2 × rings` longitudes.
///
/// # Errors
/// [`Error::Empty`] below two rings.
pub fn lumpy_sphere_mesh(rings: u32) -> Result<TriangleMesh, Error> {
    let segments = 2 * rings;
    let mut vertices = vec![lumpy(Vector3::z()), lumpy(-Vector3::z())];
    for ring in 1..rings {
        let polar = PI * f64::from(ring) / f64::from(rings);
        for segment in 0..segments {
            let azimuth = TAU * f64::from(segment) / f64::from(segments);
            vertices.push(lumpy(Vector3::new(
                polar.sin() * azimuth.cos(),
                polar.sin() * azimuth.sin(),
                polar.cos(),
            )));
        }
    }
    TriangleMesh::new(
        PointCloud::new(vertices)?,
        sphere_triangles(rings, segments),
    )
}

/// An open, gently curved patch over the unit square, `side × side` points:
/// the kind of surface a depth camera sees of a machined part.
///
/// # Errors
/// [`Error::Empty`] for a zero side.
pub fn wavy_patch(side: u32) -> Result<PointCloud, Error> {
    let step = 1.0 / f64::from(side.max(2) - 1);
    PointCloud::new(
        (0..side * side)
            .map(|index| {
                let along = f64::from(index % side) * step;
                let across = f64::from(index / side) * step;
                let height = 0.25 * (2.5 * along + 0.3).sin() * (1.7 * across).cos()
                    + 0.1 * along * across * across;
                Point::new(along, across, height)
            })
            .collect::<Vec<_>>(),
    )
}

fn lumpy(direction: Vector3<f64>) -> Point {
    let radius = 1.0
        + 0.25 * direction.x * direction.y
        + 0.2 * direction.z.powi(3)
        + 0.12 * (4.0 * direction.x + 1.0).sin();
    Point::from(direction * radius)
}

fn sphere_triangles(rings: u32, segments: u32) -> Vec<[u32; 3]> {
    let vertex = |ring: u32, segment: u32| 2 + (ring - 1) * segments + segment % segments;
    let mut triangles = Vec::new();
    for segment in 0..segments {
        triangles.push([0, vertex(1, segment), vertex(1, segment + 1)]);
        triangles.push([
            1,
            vertex(rings - 1, segment + 1),
            vertex(rings - 1, segment),
        ]);
        for ring in 1..rings - 1 {
            let (upper, lower) = (vertex(ring, segment), vertex(ring + 1, segment));
            let (upper_next, lower_next) =
                (vertex(ring, segment + 1), vertex(ring + 1, segment + 1));
            triangles.push([upper, lower, upper_next]);
            triangles.push([upper_next, lower, lower_next]);
        }
    }
    triangles
}
