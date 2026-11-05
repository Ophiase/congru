use congru::Vector;

use super::{Element, Observed};
use crate::{Camera, DepthImage};

/// How far outside a silhouette its edge is read, in pixels.
const EDGE_GAP: isize = 2;
const OUTWARD: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// The rim of the silhouette where the image jumps away too: an edge seen. Its
/// normal lies across the ray, so it says what no face can: where the object stops
/// along itself, and how it is turned on itself.
pub(super) fn contour_elements(
    observed: &Observed,
    rendered: &DepthImage,
    camera: &Camera,
    tolerance: f64,
) -> Vec<Element> {
    let mut elements = Vec::new();
    for (column, row) in rendered.positions() {
        let inside = rendered.get(column, row);
        if !inside.is_finite() || (observed.depth.get(column, row) - inside).abs() > tolerance {
            continue;
        }
        for (across, down) in OUTWARD {
            if edge_seen(observed, rendered, [column, row], [across, down], tolerance) {
                let side = inside / camera.focal;
                elements.push(Element {
                    position: camera.back_project(column, row, inside).coords,
                    normal: Vector::new(direction(across), direction(down), 0.0),
                    area: side * side,
                });
            }
        }
    }
    elements
}

/// Whether the silhouette stops next to a pixel, on this side, and the image lies
/// behind it; no return says nothing, it may be a hole. The sensor mixes pixels
/// one apart, so the image is read two out.
fn edge_seen(
    observed: &Observed,
    rendered: &DepthImage,
    [column, row]: [usize; 2],
    [across, down]: [isize; 2],
    tolerance: f64,
) -> bool {
    let reached = |gap: isize| {
        let along = column.checked_add_signed(across * gap)?;
        let downward = row.checked_add_signed(down * gap)?;
        (along < rendered.width() && downward < rendered.height()).then_some((along, downward))
    };
    let (Some(next), Some(far)) = (reached(1), reached(EDGE_GAP)) else {
        return false;
    };
    let beyond = observed.depth.get(far.0, far.1);
    !rendered.get(next.0, next.1).is_finite() && beyond > rendered.get(column, row) + tolerance
}

fn direction(sign: isize) -> f64 {
    match sign {
        1 => 1.0,
        -1 => -1.0,
        _ => 0.0,
    }
}
