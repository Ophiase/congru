use congru::Point;

use crate::pixel::coordinate;
use crate::{Camera, DepthImage};

/// What a region of interest says of a pose: the middle of the object lies in it.
/// A rectangle tested against a pose, never a cut of the data to the model's shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageBox {
    columns: [f64; 2],
    rows: [f64; 2],
}

impl ImageBox {
    /// The rectangle around `region`'s returns. A side on the border of the image
    /// stays open, since beyond it nothing was seen; `None` when there is no return.
    #[must_use]
    pub fn around(region: &DepthImage) -> Option<Self> {
        let mut returns = region
            .positions()
            .filter(|(column, row)| region.get(*column, *row).is_finite());
        let first = returns.next()?;
        let (mut left, mut right, mut top, mut bottom) = (first.0, first.0, first.1, first.1);
        for (column, row) in returns {
            left = left.min(column);
            right = right.max(column);
            top = top.min(row);
            bottom = bottom.max(row);
        }
        let side = |low: usize, high: usize, length: usize| {
            [
                if low == 0 {
                    f64::NEG_INFINITY
                } else {
                    coordinate(low)
                },
                if high + 1 >= length {
                    f64::INFINITY
                } else {
                    coordinate(high + 1)
                },
            ]
        };
        Some(Self {
            columns: side(left, right, region.width()),
            rows: side(top, bottom, region.height()),
        })
    }

    /// Whether `point`, in the camera's frame, lands in the rectangle.
    #[must_use]
    pub fn holds(&self, camera: &Camera, point: &Point) -> bool {
        camera.project(point).is_some_and(|[across, down]| {
            (self.columns[0]..=self.columns[1]).contains(&across)
                && (self.rows[0]..=self.rows[1]).contains(&down)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAMERA: Camera = Camera {
        width: 40,
        height: 40,
        focal: 40.0,
    };

    fn cut(columns: std::ops::Range<usize>) -> DepthImage {
        let mut region = DepthImage::filled(40, 40, f64::NAN);
        for column in columns {
            for row in 10..30 {
                region.set(column, row, 1.0);
            }
        }
        region
    }

    #[test]
    fn a_point_beside_the_box_is_not_held() {
        let found = ImageBox::around(&cut(10..20));
        let held = found.is_some_and(|rectangle| {
            rectangle.holds(&CAMERA, &Point::new(-0.25, 0.0, 1.0))
                && !rectangle.holds(&CAMERA, &Point::new(0.25, 0.0, 1.0))
        });
        assert!(held);
    }

    #[test]
    fn a_side_on_the_border_stays_open() {
        let found = ImageBox::around(&cut(0..20));
        assert!(
            found
                .is_some_and(|rectangle| { rectangle.holds(&CAMERA, &Point::new(-2.0, 0.0, 1.0)) })
        );
    }

    #[test]
    fn an_empty_region_says_nothing() {
        assert!(ImageBox::around(&DepthImage::filled(40, 40, f64::NAN)).is_none());
    }
}
