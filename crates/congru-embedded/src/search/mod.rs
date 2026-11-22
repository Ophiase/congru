mod build;
mod normals;
mod query;

pub use query::Nearest;

use alloc::boxed::Box;

use crate::{Error, Real, Vector3};

/// Most points a leaf scans in a row: a few comparisons more per query, far
/// fewer nodes to store.
const BUCKET: usize = 8;

/// A balanced k-d tree stored as the points themselves, reordered: the median
/// of a range is its node, children are found by arithmetic. It costs one byte
/// per inner node above the points, and queries use a fixed stack.
///
/// Indices refer to [`KdTree::points`], not to the input's order.
#[derive(Debug, Clone, PartialEq)]
pub struct KdTree<Float> {
    points: Box<[Vector3<Float>]>,
    normals: Option<Box<[Vector3<Float>]>>,
    axes: Box<[u8]>,
}

impl<Float: Real> KdTree<Float> {
    /// # Errors
    /// [`Error::Empty`], [`Error::NotFinite`], [`Error::TooManyPoints`].
    pub fn new(points: &[[Float; 3]]) -> Result<Self, Error> {
        let (order, axes) = build::arrange(points)?;
        Ok(Self {
            points: order
                .iter()
                .map(|index| Vector3(points[*index as usize]))
                .collect(),
            normals: None,
            axes,
        })
    }

    /// `normals[i]` belongs to `points[i]`, each of unit length.
    ///
    /// # Errors
    /// [`Error::LengthMismatch`], and those of [`KdTree::new`].
    pub fn with_normals(points: &[[Float; 3]], normals: &[[Float; 3]]) -> Result<Self, Error> {
        if points.len() != normals.len() {
            return Err(Error::LengthMismatch {
                left: points.len(),
                right: normals.len(),
            });
        }
        let (order, axes) = build::arrange(points)?;
        let reorder = |values: &[[Float; 3]]| -> Box<[Vector3<Float>]> {
            order
                .iter()
                .map(|index| Vector3(values[*index as usize]))
                .collect()
        };
        Ok(Self {
            points: reorder(points),
            normals: Some(reorder(normals)),
            axes,
        })
    }

    #[must_use]
    pub fn points(&self) -> &[Vector3<Float>] {
        &self.points
    }

    #[must_use]
    pub fn normals(&self) -> Option<&[Vector3<Float>]> {
        self.normals.as_deref()
    }

    /// Half the diagonal of the bounding box: the coordinate scale that
    /// rounding errors are proportional to.
    #[must_use]
    pub fn extent(&self) -> Float {
        let mut lowest = self.points[0];
        let mut highest = self.points[0];
        for point in &self.points {
            for axis in 0..3 {
                if point[axis] < lowest[axis] {
                    lowest.0[axis] = point[axis];
                }
                if point[axis] > highest[axis] {
                    highest.0[axis] = point[axis];
                }
            }
        }
        (highest - lowest).norm() * Float::of(0.5)
    }

    /// Bytes on the heap: the tree's whole cost in RAM.
    #[must_use]
    pub fn heap_bytes(&self) -> usize {
        let point = size_of::<Vector3<Float>>();
        self.points.len() * point
            + self
                .normals
                .as_ref()
                .map_or(0, |normals| normals.len() * point)
            + self.axes.len()
    }
}
