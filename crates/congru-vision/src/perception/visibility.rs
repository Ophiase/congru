use congru::Point;

use crate::pixel::{containing, coordinate};

const GRID: usize = 160;
const FIELD: f64 = 0.8;
const DEPTH_TOLERANCE: f64 = 0.01;

/// The indices of the points a sensor at the origin, looking along +z, can
/// see: projected onto a grid of directions, each cell keeps the points
/// within a centimetre of its nearest one. Self-occlusion and occluders are
/// handled alike, and a thin plate shows one face only.
#[must_use]
pub fn visible_from_origin(points: &[Point]) -> Vec<usize> {
    let cells: Vec<Option<usize>> = points.iter().map(cell_of).collect();
    let mut nearest = vec![f64::INFINITY; GRID * GRID];
    for (point, cell) in points.iter().zip(&cells) {
        if let Some(cell) = cell {
            nearest[*cell] = nearest[*cell].min(point.z);
        }
    }
    points
        .iter()
        .zip(cells)
        .enumerate()
        .filter(|(_, (point, cell))| {
            cell.is_some_and(|cell| point.z <= nearest[cell] + DEPTH_TOLERANCE)
        })
        .map(|(index, _)| index)
        .collect()
}

fn cell_of(point: &Point) -> Option<usize> {
    if point.z <= 0.0 {
        return None;
    }
    let to_cell = |slope: f64| containing((slope / FIELD + 0.5) * coordinate(GRID), GRID);
    Some(to_cell(point.y / point.z)? * GRID + to_cell(point.x / point.z)?)
}
