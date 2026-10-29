use std::num::NonZeroUsize;

use nalgebra::Unit;

use crate::search::estimate_normals;
use crate::{Distance, Error, NearestNeighbors, OrientedCloud, PointCloud, TriangleMesh, Vector};

/// What a source is aligned onto: a point cloud, its search index and,
/// for point-to-plane, its normals. Built once, it serves any number of
/// alignments, as when tracking one model over a video.
#[derive(Debug)]
pub struct Target {
    cloud: PointCloud,
    index: NearestNeighbors,
    normals: Option<Box<[Unit<Vector>]>>,
}

impl Target {
    /// A target without normals, for point-to-point only.
    ///
    /// # Errors
    /// As [`NearestNeighbors::new`].
    pub fn new(cloud: PointCloud) -> Result<Self, Error> {
        let index = NearestNeighbors::new(&cloud)?;
        Ok(Self {
            cloud,
            index,
            normals: None,
        })
    }

    /// A target whose normals are fitted to each point's `neighbors` nearest
    /// points; 8 to 20 suits most scans.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] when `neighbors` is below 3 or above the
    /// point count.
    pub fn with_estimated_normals(cloud: PointCloud, neighbors: usize) -> Result<Self, Error> {
        let mut target = Self::new(cloud)?;
        let neighbors = NonZeroUsize::new(neighbors).ok_or(Error::InvalidParameter {
            name: "normal neighbors",
            expected: "at least 3",
            value: 0.0,
        })?;
        target.normals = Some(estimate_normals(&target.cloud, &target.index, neighbors)?);
        Ok(target)
    }

    /// A target whose normals are known, from a mesh or a sensor.
    ///
    /// # Errors
    /// As [`NearestNeighbors::new`].
    pub fn from_oriented(oriented: OrientedCloud) -> Result<Self, Error> {
        let (cloud, normals) = oriented.into_parts();
        let mut target = Self::new(cloud)?;
        target.normals = Some(normals);
        Ok(target)
    }

    /// A mesh, sampled every `spacing` with face normals; see
    /// [`TriangleMesh::sample_surface`]. A spacing below the source's noise
    /// makes point-to-plane close to true point-to-surface distance.
    ///
    /// # Errors
    /// As [`TriangleMesh::sample_surface`].
    pub fn from_mesh(mesh: &TriangleMesh, spacing: Distance) -> Result<Self, Error> {
        Self::from_oriented(mesh.sample_surface(spacing)?)
    }

    #[must_use]
    pub const fn cloud(&self) -> &PointCloud {
        &self.cloud
    }

    #[must_use]
    pub fn normals(&self) -> Option<&[Unit<Vector>]> {
        self.normals.as_deref()
    }

    #[must_use]
    pub const fn index(&self) -> &NearestNeighbors {
        &self.index
    }
}
