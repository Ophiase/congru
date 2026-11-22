use nalgebra::Unit;

use super::{PointCloud, Vector};
use crate::Error;

/// A point cloud with one unit normal per point, the sign of each normal
/// left free.
#[derive(Debug, Clone, PartialEq)]
pub struct OrientedCloud {
    cloud: PointCloud,
    normals: Box<[Unit<Vector>]>,
}

impl OrientedCloud {
    /// Normals are normalised here; their length does not matter.
    ///
    /// # Errors
    /// [`Error::LengthMismatch`], or [`Error::InvalidParameter`] for a
    /// normal of zero or non-finite length.
    pub fn new(cloud: PointCloud, normals: &[Vector]) -> Result<Self, Error> {
        if normals.len() != cloud.count() {
            return Err(Error::LengthMismatch {
                what: "points and normals",
                left: cloud.count(),
                right: normals.len(),
            });
        }
        let normals = normals
            .iter()
            .map(|normal| {
                Unit::try_new(*normal, f64::EPSILON).ok_or(Error::InvalidParameter {
                    name: "normal",
                    expected: "of finite, non-zero length",
                    value: normal.norm(),
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { cloud, normals })
    }

    pub(crate) fn from_unit(cloud: PointCloud, normals: Box<[Unit<Vector>]>) -> Self {
        Self { cloud, normals }
    }

    #[must_use]
    pub const fn cloud(&self) -> &PointCloud {
        &self.cloud
    }

    #[must_use]
    pub fn normals(&self) -> &[Unit<Vector>] {
        &self.normals
    }

    pub(crate) fn into_parts(self) -> (PointCloud, Box<[Unit<Vector>]>) {
        (self.cloud, self.normals)
    }
}
