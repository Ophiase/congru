use recalage::Point;

use crate::pixel::coordinate as as_coordinate;

/// A pinhole camera with square pixels and the principal point at the
/// centre of the image. Camera frame: x to the right, y down, z forward,
/// in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub width: usize,
    pub height: usize,
    /// In pixels.
    pub focal: f64,
}

impl Camera {
    #[must_use]
    pub fn with_field_of_view(width: usize, height: usize, horizontal_degrees: f64) -> Self {
        Self {
            width,
            height,
            focal: as_coordinate(width) / 2.0 / (horizontal_degrees.to_radians() / 2.0).tan(),
        }
    }

    /// Where a point of the camera frame lands on the image, in continuous
    /// pixel coordinates; `None` behind the camera.
    #[must_use]
    pub fn project(&self, point: &Point) -> Option<[f64; 2]> {
        (point.z > 0.0).then(|| {
            [
                self.focal * point.x / point.z + as_coordinate(self.width) / 2.0,
                self.focal * point.y / point.z + as_coordinate(self.height) / 2.0,
            ]
        })
    }

    /// The point of the camera frame seen at the centre of a pixel, at
    /// `depth` along the optical axis.
    #[must_use]
    pub fn back_project(&self, column: usize, row: usize, depth: f64) -> Point {
        let across = (as_coordinate(column) + 0.5 - as_coordinate(self.width) / 2.0) / self.focal;
        let down = (as_coordinate(row) + 0.5 - as_coordinate(self.height) / 2.0) / self.focal;
        Point::new(across * depth, down * depth, depth)
    }
}
