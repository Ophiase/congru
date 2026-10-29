use nalgebra::Vector3;
use recalage::RigidTransform;
use recalage_vision::{DepthImage, Image};

use super::stage::Stage;
use crate::picture::palette::{EMPTY, ESTIMATE, Tone, grey};

const AMBIENT: f64 = 0.15;

/// What a panel shows: the degraded depth as a relief, the part's outline
/// at the estimated pose over it.
pub fn panel(stage: &Stage, depth: &DepthImage, estimate: Option<&RigidTransform>) -> Image<Tone> {
    let mut picture = relief(depth, stage.camera.focal);
    if let Some(pose) = estimate {
        let silhouette = stage.shoot(pose, false).depth.map(f64::is_finite);
        for (column, row) in silhouette.positions() {
            if on_outline(&silhouette, column, row) {
                picture.set(column, row, ESTIMATE);
            }
        }
    }
    picture
}

/// Shaded by the surface normal the depth itself implies, lit from the
/// camera: noise shows as grain, blur as soft edges, holes as black.
fn relief(depth: &DepthImage, focal: f64) -> Image<Tone> {
    let mut picture = Image::filled(depth.width(), depth.height(), EMPTY);
    for (column, row) in depth
        .positions()
        .filter(|(column, row)| *column > 0 && *row > 0)
    {
        let here = depth.get(column, row);
        let (left, above) = (depth.get(column - 1, row), depth.get(column, row - 1));
        if !(here.is_finite() && left.is_finite() && above.is_finite()) {
            continue;
        }
        let pixel = here / focal;
        let normal = Vector3::new((here - left) / pixel, (here - above) / pixel, -1.0).normalize();
        picture.set(column, row, grey(AMBIENT + (1.0 - AMBIENT) * -normal.z));
    }
    picture
}

fn on_outline(silhouette: &Image<bool>, column: usize, row: usize) -> bool {
    if !silhouette.get(column, row) {
        return false;
    }
    let last_column = silhouette.width() - 1;
    let last_row = silhouette.height() - 1;
    [
        (column.saturating_sub(1), row),
        ((column + 1).min(last_column), row),
        (column, row.saturating_sub(1)),
        (column, (row + 1).min(last_row)),
    ]
    .iter()
    .any(|(other_column, other_row)| !silhouette.get(*other_column, *other_row))
}
