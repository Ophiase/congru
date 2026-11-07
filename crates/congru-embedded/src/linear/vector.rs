use core::ops::{Add, AddAssign, Index, Mul, Neg, Sub, SubAssign};

use crate::Real;

/// Three coordinates, in the array a sensor buffer already holds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(transparent)]
pub struct Vector3<Float>(pub [Float; 3]);

impl<Float: Real> Vector3<Float> {
    pub const ZERO: Self = Self([Float::ZERO; 3]);

    #[must_use]
    #[inline]
    pub fn dot(self, other: Self) -> Float {
        self.0[0] * other.0[0] + self.0[1] * other.0[1] + self.0[2] * other.0[2]
    }

    #[must_use]
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        let ([first, second, third], [one, two, three]) = (self.0, other.0);
        Self([
            second * three - third * two,
            third * one - first * three,
            first * two - second * one,
        ])
    }

    #[must_use]
    #[inline]
    pub fn squared_norm(self) -> Float {
        self.dot(self)
    }

    #[must_use]
    #[inline]
    pub fn norm(self) -> Float {
        self.squared_norm().root()
    }

    #[must_use]
    #[inline]
    pub fn finite(self) -> bool {
        self.0.iter().all(|coordinate| coordinate.finite())
    }

    /// The outer product `self · otherᵀ`, by rows.
    #[inline]
    pub(crate) fn outer(self, other: Self) -> [[Float; 3]; 3] {
        self.0.map(|left| other.0.map(|right| left * right))
    }
}

impl<Float> From<[Float; 3]> for Vector3<Float> {
    #[inline]
    fn from(coordinates: [Float; 3]) -> Self {
        Self(coordinates)
    }
}

impl<Float> Index<usize> for Vector3<Float> {
    type Output = Float;

    #[inline]
    fn index(&self, axis: usize) -> &Float {
        &self.0[axis]
    }
}

impl<Float: Real> Add for Vector3<Float> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl<Float: Real> Sub for Vector3<Float> {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

impl<Float: Real> Mul<Float> for Vector3<Float> {
    type Output = Self;

    #[inline]
    fn mul(self, factor: Float) -> Self {
        Self(self.0.map(|coordinate| coordinate * factor))
    }
}

impl<Float: Real> Neg for Vector3<Float> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self(self.0.map(|coordinate| -coordinate))
    }
}

impl<Float: Real> AddAssign for Vector3<Float> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<Float: Real> SubAssign for Vector3<Float> {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
