use super::{Point, RigidTransform};
use crate::Error;

/// A non-empty set of finite 3D points.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloud {
    points: Box<[Point]>,
}

impl PointCloud {
    /// # Errors
    /// [`Error::Empty`] or [`Error::NonFinite`].
    pub fn new(points: impl Into<Vec<Point>>) -> Result<Self, Error> {
        let points: Vec<Point> = points.into();
        if points.is_empty() {
            return Err(Error::Empty {
                what: "point cloud",
            });
        }
        if let Some(index) = points
            .iter()
            .position(|point| !point.coords.iter().all(|coordinate| coordinate.is_finite()))
        {
            return Err(Error::NonFinite { index });
        }
        Ok(Self {
            points: points.into_boxed_slice(),
        })
    }

    /// From interleaved coordinates `x0 y0 z0 x1 y1 z1 …`, the layout of an
    /// `(n, 3)` C-contiguous array.
    ///
    /// # Errors
    /// [`Error::Ragged`], [`Error::Empty`] or [`Error::NonFinite`].
    pub fn from_xyz(coordinates: &[f64]) -> Result<Self, Error> {
        if !coordinates.len().is_multiple_of(3) {
            return Err(Error::Ragged {
                length: coordinates.len(),
            });
        }
        Self::new(
            coordinates
                .as_chunks::<3>()
                .0
                .iter()
                .map(|&row| Point::from(row))
                .collect::<Vec<_>>(),
        )
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// How many points; never zero.
    #[must_use]
    pub fn count(&self) -> usize {
        self.points.len()
    }

    #[must_use]
    pub fn centroid(&self) -> Point {
        mean(&self.points)
    }

    #[must_use]
    pub fn transformed(&self, transform: &RigidTransform) -> Self {
        Self {
            points: self
                .points
                .iter()
                .map(|point| transform.apply(point))
                .collect(),
        }
    }

    /// Interleaved coordinates, the inverse of [`PointCloud::from_xyz`].
    #[must_use]
    pub fn to_xyz(&self) -> Box<[f64]> {
        self.points
            .iter()
            .flat_map(|point| point.coords.iter().copied())
            .collect()
    }
}

pub(crate) fn mean(points: &[Point]) -> Point {
    let sum = points
        .iter()
        .fold(nalgebra::Vector3::zeros(), |sum, point| sum + point.coords);
    Point::from(sum / count_as_float(points.len()))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "point counts stay far below 2^52"
)]
pub(crate) fn count_as_float(count: usize) -> f64 {
    count as f64
}
