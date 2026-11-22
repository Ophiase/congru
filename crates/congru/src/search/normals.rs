use std::num::NonZeroUsize;

use nalgebra::{Matrix3, Unit};

use super::{NearestNeighbors, map};
use crate::geometry::mean;
use crate::{Error, Point, PointCloud, Vector};

const MINIMUM_NEIGHBORS: usize = 3;

/// The normal of the plane fitted to each point's `neighbors` nearest
/// points (the point itself included): the eigenvector of their covariance
/// with the smallest eigenvalue.
pub(crate) fn estimate_normals(
    cloud: &PointCloud,
    index: &NearestNeighbors,
    neighbors: NonZeroUsize,
) -> Result<Box<[Unit<Vector>]>, Error> {
    if neighbors.get() < MINIMUM_NEIGHBORS || neighbors.get() > cloud.count() {
        return Err(Error::InvalidParameter {
            name: "normal neighbors",
            expected: "at least 3 and at most the point count",
            value: crate::geometry::count_as_float(neighbors.get()),
        });
    }
    let normals = map(cloud.points(), |point| {
        let patch: Vec<Point> = index
            .k_nearest(point, neighbors)
            .iter()
            .map(|neighbor| cloud.points()[neighbor.index])
            .collect();
        plane_normal(&patch)
    });
    Ok(normals.into_boxed_slice())
}

fn plane_normal(patch: &[Point]) -> Unit<Vector> {
    let centre = mean(patch);
    let covariance = patch.iter().fold(Matrix3::zeros(), |sum, point| {
        let offset = point - centre;
        sum + offset * offset.transpose()
    });
    let eigen = covariance.symmetric_eigen();
    let smallest = eigen.eigenvalues.imin();
    Unit::try_new(
        eigen.eigenvectors.column(smallest).into_owned(),
        f64::EPSILON,
    )
    .unwrap_or_else(Vector::z_axis)
}
