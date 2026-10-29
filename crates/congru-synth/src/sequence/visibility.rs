use congru::Point;

use crate::casts::{as_count, as_float};

const GRID: usize = 160;
const FIELD: f64 = 0.8;
const DEPTH_TOLERANCE: f64 = 0.01;

/// The points the sensor at the origin can see: projected onto a grid of
/// directions, each cell keeps the points within a small depth of its
/// nearest one. Self-occlusion and occluders are handled alike.
pub(super) fn visible(scene: &[Point]) -> Vec<Point> {
    let mut nearest = vec![f64::INFINITY; GRID * GRID];
    let cells: Vec<Option<usize>> = scene.iter().map(cell_of).collect();
    for (point, cell) in scene.iter().zip(&cells) {
        if let Some(cell) = cell {
            nearest[*cell] = nearest[*cell].min(point.z);
        }
    }
    scene
        .iter()
        .zip(cells)
        .filter_map(|(point, cell)| {
            cell.filter(|cell| point.z <= nearest[*cell] + DEPTH_TOLERANCE)
                .map(|_| *point)
        })
        .collect()
}

fn cell_of(point: &Point) -> Option<usize> {
    if point.z <= 0.0 {
        return None;
    }
    let to_cell = |slope: f64| {
        let position = (slope / FIELD + 0.5) * as_float(GRID);
        (0.0..as_float(GRID))
            .contains(&position)
            .then(|| as_count(position.floor()))
    };
    Some(to_cell(point.y / point.z)? * GRID + to_cell(point.x / point.z)?)
}
