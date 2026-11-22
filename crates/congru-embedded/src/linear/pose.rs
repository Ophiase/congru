use super::{UnitQuaternion, Vector3};
use crate::Real;

/// A rigid motion: `point ↦ rotation · point + translation`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose<Float> {
    pub rotation: UnitQuaternion<Float>,
    pub translation: Vector3<Float>,
}

impl<Float: Real> Pose<Float> {
    #[must_use]
    pub fn identity() -> Self {
        Self {
            rotation: UnitQuaternion::identity(),
            translation: Vector3::ZERO,
        }
    }

    /// This motion first, then `next`.
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        Self {
            rotation: self.rotation.then(next.rotation),
            translation: next.frozen().apply(self.translation),
        }
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        let rotation = self.rotation.inverse();
        let turned = Frozen::rotation_of(rotation).apply(self.translation);
        Self {
            rotation,
            translation: -turned,
        }
    }

    #[must_use]
    pub fn apply(&self, point: Vector3<Float>) -> Vector3<Float> {
        self.frozen().apply(point)
    }

    /// The angle, in radians, of the rotation left between the two poses.
    #[must_use]
    pub fn rotation_error_to(&self, other: &Self) -> Float {
        self.rotation.inverse().then(other.rotation).angle()
    }

    #[must_use]
    pub fn translation_error_to(&self, other: &Self) -> Float {
        (self.translation - other.translation).norm()
    }

    #[inline]
    pub(crate) fn frozen(&self) -> Frozen<Float> {
        Frozen {
            matrix: self.rotation.matrix(),
            translation: self.translation,
        }
    }
}

/// A pose as a matrix, computed once and applied to many points.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Frozen<Float> {
    matrix: [[Float; 3]; 3],
    translation: Vector3<Float>,
}

impl<Float: Real> Frozen<Float> {
    fn rotation_of(rotation: UnitQuaternion<Float>) -> Self {
        Self {
            matrix: rotation.matrix(),
            translation: Vector3::ZERO,
        }
    }

    #[inline]
    pub(crate) fn apply(&self, point: Vector3<Float>) -> Vector3<Float> {
        Vector3(self.matrix.map(|row| Vector3(row).dot(point))) + self.translation
    }
}
