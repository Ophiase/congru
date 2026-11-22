use std::f64::consts::TAU;

use congru::{Error, Point, PointCloud, TriangleMesh};

const PERIOD_FRAMES: f64 = 40.0;

/// The mesh flexed at `frame`: each coordinate shifted by up to `amplitude`
/// with the square of another, swinging back and forth, so the warp has no
/// rotation in it to first order: a part that is not perfectly rigid.
///
/// # Errors
/// Never in practice; the type is the mesh constructor's.
pub fn flexed(
    mesh: &TriangleMesh,
    amplitude: f64,
    span: f64,
    frame: usize,
) -> Result<TriangleMesh, Error> {
    let swing = (TAU * congru_vision::pixel::coordinate(frame) / PERIOD_FRAMES).sin() * amplitude;
    let half = span / 2.0;
    let square = |coordinate: f64| (coordinate / half).powi(2);
    let moved: Vec<Point> = mesh
        .vertices()
        .points()
        .iter()
        .map(|point| {
            point
                + nalgebra::Vector3::new(square(point.y), square(point.z), square(point.x)) * swing
        })
        .collect();
    TriangleMesh::new(PointCloud::new(moved)?, mesh.triangles().to_vec())
}
