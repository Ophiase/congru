use congru::Point;
use congru_vision::pixel::containing;
use congru_vision::{Camera, DepthImage};

/// Points back into a depth image, the nearest per pixel, so every step is
/// drawn the same way.
pub fn splat(points: &[Point], camera: &Camera) -> DepthImage {
    let mut depth = DepthImage::filled(camera.width, camera.height, f64::NAN);
    for point in points {
        let Some([across, down]) = camera.project(point) else {
            continue;
        };
        if let (Some(column), Some(row)) = (
            containing(across, camera.width),
            containing(down, camera.height),
        ) {
            let current = depth.get(column, row);
            if current.is_nan() || point.z < current {
                depth.set(column, row, point.z);
            }
        }
    }
    depth
}
