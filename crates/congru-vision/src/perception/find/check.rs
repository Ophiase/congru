use congru::{Point, RigidTransform, TriangleMesh};

use crate::pixel::{containing, coordinate};
use crate::{Camera, DepthImage, render};

/// How far outside the silhouette its edge is read, in pixels.
const EDGE_GAP: usize = 2;

/// How well `depth` agrees with the model rendered at `pose`. Inside the
/// silhouette, pixels seen at the model's depth count for it and pixels
/// seen behind it count against it. Just outside, a true object stands
/// out: the frame jumps away; a model sunk into a wall or a table does not,
/// the surface goes on. The two shares are multiplied.
pub(super) fn agreement(
    depth: &DepthImage,
    camera: &Camera,
    model: &TriangleMesh,
    pose: &RigidTransform,
    tolerance: f64,
) -> f64 {
    let rendered = render(camera, &[(model, *pose)]).depth;
    let (mut covered, mut agree, mut behind) = (0_usize, 0_usize, 0_usize);
    let (mut standing, mut flush) = (0_usize, 0_usize);
    for (column, row) in rendered.positions() {
        let seen = depth.get(column, row);
        let expected = rendered.get(column, row);
        if expected.is_finite() {
            covered += 1;
            if (seen - expected).abs() <= tolerance {
                agree += 1;
            } else if seen > expected + tolerance {
                behind += 1;
            }
        } else if let Some(edge) = edge_depth(&rendered, column, row) {
            if seen.is_nan() || seen > edge + tolerance {
                standing += 1;
            } else if (seen - edge).abs() <= tolerance {
                flush += 1;
            }
        }
    }
    let inside = (coordinate(agree) - coordinate(behind)) / coordinate(covered.max(1));
    let outline = coordinate(standing) / coordinate((standing + flush).max(1));
    inside * outline
}

/// The share of `points`, carried by `pose`, that the camera sees inside its
/// image: a pose mostly out of the frame is not judged on what remains.
pub(super) fn share_in_frame<'a>(
    camera: &Camera,
    pose: &RigidTransform,
    points: impl ExactSizeIterator<Item = &'a Point>,
) -> f64 {
    let total = points.len();
    let inside = points
        .filter(|point| {
            camera
                .project(&pose.apply(point))
                .is_some_and(|[across, down]| {
                    containing(across, camera.width).is_some()
                        && containing(down, camera.height).is_some()
                })
        })
        .count();
    coordinate(inside) / coordinate(total.max(1))
}

/// The model's depth at a silhouette pixel a few pixels from this one.
fn edge_depth(rendered: &DepthImage, column: usize, row: usize) -> Option<f64> {
    [
        (column.checked_sub(EDGE_GAP), Some(row)),
        (Some(column + EDGE_GAP), Some(row)),
        (Some(column), row.checked_sub(EDGE_GAP)),
        (Some(column), Some(row + EDGE_GAP)),
    ]
    .into_iter()
    .filter_map(|(across, down)| {
        let (across, down) = (across?, down?);
        (across < rendered.width() && down < rendered.height())
            .then(|| rendered.get(across, down))
            .filter(|value| value.is_finite())
    })
    .reduce(f64::min)
}
