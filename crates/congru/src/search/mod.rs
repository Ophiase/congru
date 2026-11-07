mod normals;
mod parallel;
mod surface;

use std::num::NonZeroUsize;

use kiddo::{ImmutableKdTree, SquaredEuclidean};

use crate::{Error, Point, PointCloud};

pub(crate) use normals::estimate_normals;
pub use parallel::map;
pub use surface::{Closest, DistanceField, Grid, MeshIndex, Surface};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Neighbor {
    pub index: usize,
    pub squared_distance: f64,
}

/// A k-d tree over a point cloud, built once, queried many times.
pub struct NearestNeighbors {
    tree: ImmutableKdTree<f64, 3>,
}

impl NearestNeighbors {
    /// # Errors
    /// [`Error::InvalidParameter`] past `u32::MAX` points.
    pub fn new(cloud: &PointCloud) -> Result<Self, Error> {
        let rows: Vec<[f64; 3]> = cloud
            .points()
            .iter()
            .map(|point| point.coords.into())
            .collect();
        ImmutableKdTree::new_from_slice(&rows)
            .map(|tree| Self { tree })
            .map_err(|_| Error::InvalidParameter {
                name: "point count",
                expected: "at most u32::MAX",
                value: crate::geometry::count_as_float(rows.len()),
            })
    }

    #[must_use]
    pub fn nearest(&self, query: &Point) -> Neighbor {
        let found = self
            .tree
            .query(&query.coords.into())
            .nearest_one::<SquaredEuclidean<f64>>()
            .execute();
        Neighbor {
            index: found.item as usize,
            squared_distance: found.distance,
        }
    }

    /// The `count` closest indexed points, nearest first.
    #[must_use]
    pub fn k_nearest(&self, query: &Point, count: NonZeroUsize) -> Vec<Neighbor> {
        self.tree
            .query(&query.coords.into())
            .nearest_n::<SquaredEuclidean<f64>>(count)
            .execute()
            .into_iter()
            .map(|found| Neighbor {
                index: found.item as usize,
                squared_distance: found.distance,
            })
            .collect()
    }

    /// Every indexed point within `radius`, in no order.
    #[must_use]
    pub fn within(&self, query: &Point, radius: f64) -> Vec<Neighbor> {
        self.tree
            .query(&query.coords.into())
            .within::<SquaredEuclidean<f64>>(radius * radius)
            .unsorted()
            .execute()
            .into_iter()
            .map(|found| Neighbor {
                index: found.item as usize,
                squared_distance: found.distance,
            })
            .collect()
    }
}

impl std::fmt::Debug for NearestNeighbors {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NearestNeighbors")
            .field("size", &self.tree.size())
            .finish()
    }
}
