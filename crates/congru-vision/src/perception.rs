use congru::Point;

use crate::{Camera, DepthImage};

/// Every pixel that returned a depth, as a point of the camera frame.
#[must_use]
pub fn back_project(depth: &DepthImage, camera: &Camera) -> Vec<Point> {
    depth
        .positions()
        .filter_map(|(column, row)| {
            let value = depth.get(column, row);
            value
                .is_finite()
                .then(|| camera.back_project(column, row, value))
        })
        .collect()
}

/// The points within `radius` of `centre`: what is left of a frame once the
/// table and the background are cut away around a predicted position.
#[must_use]
pub fn within(points: &[Point], centre: &Point, radius: f64) -> Vec<Point> {
    let limit = radius * radius;
    points
        .iter()
        .filter(|point| (*point - centre).norm_squared() <= limit)
        .copied()
        .collect()
}
