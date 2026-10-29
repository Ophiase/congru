use nalgebra::{Isometry3, Matrix4, Translation3, UnitQuaternion};

use super::{Point, Vector};
use crate::Error;

const ORTHONORMAL_TOLERANCE: f64 = 1e-6;

/// A proper rigid motion `p ↦ R p + t`: a rotation, then a translation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidTransform(Isometry3<f64>);

impl RigidTransform {
    #[must_use]
    pub fn identity() -> Self {
        Self(Isometry3::identity())
    }

    #[must_use]
    pub fn from_parts(rotation: UnitQuaternion<f64>, translation: impl Into<Vector>) -> Self {
        Self(Isometry3::from_parts(
            Translation3::from(translation.into()),
            rotation,
        ))
    }

    /// From a rotation vector (axis times angle, in radians) and a translation.
    #[must_use]
    pub fn from_axis_angle(axis_angle: Vector, translation: impl Into<Vector>) -> Self {
        Self::from_parts(UnitQuaternion::from_scaled_axis(axis_angle), translation)
    }

    /// From a 4×4 homogeneous matrix.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] when the matrix is not a proper rigid
    /// motion within `1e-6`: last row `0 0 0 1`, orthonormal rotation block,
    /// determinant `+1`.
    pub fn from_matrix(matrix: &Matrix4<f64>) -> Result<Self, Error> {
        let rotation = matrix.fixed_view::<3, 3>(0, 0).into_owned();
        let drift = (rotation.transpose() * rotation - nalgebra::Matrix3::identity()).norm()
            + (matrix.row(3) - nalgebra::RowVector4::new(0.0, 0.0, 0.0, 1.0)).norm();
        if !(drift < ORTHONORMAL_TOLERANCE && rotation.determinant() > 0.0) {
            return Err(Error::InvalidParameter {
                name: "transform",
                expected: "a proper rigid motion",
                value: drift,
            });
        }
        let rotation = nalgebra::Rotation3::from_matrix_unchecked(rotation);
        Ok(Self::from_parts(
            UnitQuaternion::from_rotation_matrix(&rotation),
            matrix.fixed_view::<3, 1>(0, 3).into_owned(),
        ))
    }

    /// From a row-major 4×4 matrix, the layout of a C `double[16]` or a NumPy
    /// `(4, 4)` array.
    ///
    /// # Errors
    /// As [`RigidTransform::from_matrix`].
    pub fn from_row_major(entries: &[f64; 16]) -> Result<Self, Error> {
        Self::from_matrix(&Matrix4::from_row_slice(entries))
    }

    #[must_use]
    pub fn to_matrix(&self) -> Matrix4<f64> {
        self.0.to_homogeneous()
    }

    /// The 4×4 homogeneous matrix, row-major.
    #[must_use]
    pub fn to_row_major(&self) -> [f64; 16] {
        let mut entries = [0.0; 16];
        for (entry, value) in entries.iter_mut().zip(self.to_matrix().transpose().iter()) {
            *entry = *value;
        }
        entries
    }

    #[must_use]
    pub fn rotation(&self) -> UnitQuaternion<f64> {
        self.0.rotation
    }

    #[must_use]
    pub fn translation(&self) -> Vector {
        self.0.translation.vector
    }

    #[must_use]
    pub const fn isometry(&self) -> &Isometry3<f64> {
        &self.0
    }

    #[must_use]
    pub fn apply(&self, point: &Point) -> Point {
        self.0 * point
    }

    /// `self`, then `next`.
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        Self(next.0 * self.0)
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }

    /// Angle, in radians, of the rotation between `self` and `other`.
    #[must_use]
    pub fn rotation_error_to(&self, other: &Self) -> f64 {
        self.0.rotation.angle_to(&other.0.rotation)
    }

    /// Distance between the translations of `self` and `other`.
    #[must_use]
    pub fn translation_error_to(&self, other: &Self) -> f64 {
        (self.translation() - other.translation()).norm()
    }
}

impl Default for RigidTransform {
    fn default() -> Self {
        Self::identity()
    }
}
