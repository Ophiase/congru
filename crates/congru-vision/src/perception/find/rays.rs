use congru::{Point, RigidTransform, fit_rigid};

use super::pairs::Oriented;
use crate::pixel::containing;
use crate::{Camera, DepthImage};

/// A wide gate gives the basin, a narrow one the precision: each pass of a
/// refinement divides the first gate by this, down to the last.
pub(super) const GATE_SHRINKS: [f64; 4] = [1.0, 1.5, 2.5, 4.0];
/// Fewer pairs than this and the motion is not trusted.
const LEAST_PAIRS: usize = 12;
/// How many pixels around its own a model point looks for its nearest seen
/// point: enough to slide sideways, too few to reach another surface.
const WINDOW: usize = 3;

/// `pose` brought onto `depth` along the camera's rays: each model point
/// facing the camera reads only the pixels around its own, so the surfaces
/// around the object cannot pull it. Pairs farther apart than `gate` are
/// left out: a pixel seen in front is a shadow, one seen behind is void.
pub(super) fn along_rays(
    depth: &DepthImage,
    camera: &Camera,
    points: &[Oriented],
    pose: &RigidTransform,
    gate: f64,
    iterations: u32,
) -> RigidTransform {
    let mut pose = *pose;
    for _ in 0..iterations {
        let (model, seen): (Vec<Point>, Vec<Point>) = points
            .iter()
            .filter_map(|oriented| hit(depth, camera, &pose, oriented, gate))
            .unzip();
        if model.len() < LEAST_PAIRS {
            break;
        }
        match fit_rigid(&model, &seen, None) {
            Ok(fit) => pose = pose.then(&fit.transform),
            Err(_) => break,
        }
    }
    pose
}

fn hit(
    depth: &DepthImage,
    camera: &Camera,
    pose: &RigidTransform,
    oriented: &Oriented,
    gate: f64,
) -> Option<(Point, Point)> {
    let placed = pose.apply(&oriented.point);
    let normal = pose.rotation() * oriented.normal;
    if normal.dot(&placed.coords) >= 0.0 {
        return None;
    }
    let [across, down] = camera.project(&placed)?;
    let (column, row) = (
        containing(across, camera.width)?,
        containing(down, camera.height)?,
    );
    let columns = column.saturating_sub(WINDOW)..=(column + WINDOW).min(camera.width - 1);
    let rows = row.saturating_sub(WINDOW)..=(row + WINDOW).min(camera.height - 1);
    rows.flat_map(|row| columns.clone().map(move |column| (column, row)))
        .filter_map(|(column, row)| {
            let value = depth.get(column, row);
            value
                .is_finite()
                .then(|| camera.back_project(column, row, value))
        })
        .map(|seen| ((seen - placed).norm(), seen))
        .filter(|(distance, _)| *distance <= gate)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, seen)| (placed, seen))
}
