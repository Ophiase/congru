use crate::pixel::{containing, coordinate};
use crate::{Camera, DepthImage, Image};
use congru::{Point, RigidTransform, TriangleMesh};

const NEAR: f64 = 0.05;
const AMBIENT: f64 = 40.0;
const DIFFUSE: f64 = 200.0;

/// A rendered view: the depth, and a grey image shaded by how squarely each
/// face looks at the camera.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub depth: DepthImage,
    pub grey: Image<u8>,
}

/// Each mesh placed in the camera frame by its pose, rasterised into one
/// depth buffer: every pixel keeps the nearest surface.
#[must_use]
pub fn render(camera: &Camera, scene: &[(&TriangleMesh, RigidTransform)]) -> Frame {
    let mut frame = Frame {
        depth: Image::filled(camera.width, camera.height, f64::NAN),
        grey: Image::filled(camera.width, camera.height, 0),
    };
    for (mesh, pose) in scene {
        let placed: Vec<Point> = mesh
            .vertices()
            .points()
            .iter()
            .map(|vertex| pose.apply(vertex))
            .collect();
        for corners in mesh.triangles() {
            let triangle = corners.map(|corner| placed[corner as usize]);
            draw(&mut frame, camera, &triangle);
        }
    }
    frame
}

fn draw(frame: &mut Frame, camera: &Camera, triangle: &[Point; 3]) {
    if triangle.iter().any(|corner| corner.z <= NEAR) {
        return;
    }
    let projected = triangle.map(|corner| camera.project(&corner).unwrap_or([f64::NAN; 2]));
    let area = edge(&projected[0], &projected[1], &projected[2]);
    if area.abs() < f64::EPSILON || area.is_nan() {
        return;
    }
    let tone = shade(triangle);
    for (column, row) in covered(camera, &projected) {
        let centre = [coordinate(column) + 0.5, coordinate(row) + 0.5];
        let weights = [
            edge(&projected[1], &projected[2], &centre) / area,
            edge(&projected[2], &projected[0], &centre) / area,
            edge(&projected[0], &projected[1], &centre) / area,
        ];
        if weights.iter().any(|weight| *weight < 0.0) {
            continue;
        }
        let inverse: f64 = weights
            .iter()
            .zip(triangle)
            .map(|(weight, corner)| weight / corner.z)
            .sum();
        let depth = 1.0 / inverse;
        let current = frame.depth.get(column, row);
        if current.is_nan() || depth < current {
            frame.depth.set(column, row, depth);
            frame.grey.set(column, row, tone);
        }
    }
}

fn covered(
    camera: &Camera,
    projected: &[[f64; 2]; 3],
) -> impl Iterator<Item = (usize, usize)> + use<> {
    let span = |axis: usize, size: usize| {
        let low = projected
            .iter()
            .map(|corner| corner[axis])
            .fold(f64::INFINITY, f64::min);
        let high = projected
            .iter()
            .map(|corner| corner[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        let last = coordinate(size) - 0.5;
        match (
            containing(low.max(0.0), size),
            containing(high.min(last), size),
        ) {
            (Some(first), Some(final_index)) => first..final_index + 1,
            _ => 0..0,
        }
    };
    let columns = span(0, camera.width);
    let rows = span(1, camera.height);
    rows.flat_map(move |row| columns.clone().map(move |column| (column, row)))
}

fn edge(from: &[f64; 2], to: &[f64; 2], point: &[f64; 2]) -> f64 {
    (to[0] - from[0]) * (point[1] - from[1]) - (to[1] - from[1]) * (point[0] - from[0])
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a tone clamped to 0..=255 first"
)]
fn shade(triangle: &[Point; 3]) -> u8 {
    let normal = (triangle[1] - triangle[0]).cross(&(triangle[2] - triangle[0]));
    let centre = (triangle[0].coords + triangle[1].coords + triangle[2].coords) / 3.0;
    let facing = (normal.dot(&centre) / (normal.norm() * centre.norm())).abs();
    (AMBIENT + DIFFUSE * facing).clamp(0.0, 255.0) as u8
}
