use core::fmt::Debug;
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// The scalar of every computation: `f32` where doubles are emulated, `f64`
/// where they are not. Elementary functions come from `libm`, so nothing here
/// needs the standard library.
pub trait Real:
    Copy
    + Debug
    + Default
    + PartialOrd
    + Send
    + Sync
    + 'static
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
{
    const ZERO: Self;
    const ONE: Self;
    const INFINITY: Self;
    /// Spacing of the representable numbers just above one: a coordinate of
    /// magnitude `m` is known to `m · EPSILON / 2` at best.
    const EPSILON: Self;

    /// Nearest representable value; for constants, folded at compile time.
    #[must_use]
    fn of(value: f64) -> Self;
    #[must_use]
    fn of_count(count: usize) -> Self;
    #[must_use]
    fn as_f64(self) -> f64;
    #[must_use]
    fn root(self) -> Self;
    #[must_use]
    fn magnitude(self) -> Self;
    /// Angle of the vector `(along, self)`, in radians.
    #[must_use]
    fn angle_over(self, along: Self) -> Self;
    #[must_use]
    fn finite(self) -> bool;

    /// False for NaN and infinity, unlike a bare comparison.
    #[must_use]
    #[inline]
    fn strictly_positive(self) -> bool {
        self > Self::ZERO && self.finite()
    }

    #[must_use]
    #[inline]
    fn larger(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl Real for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const INFINITY: Self = Self::INFINITY;
    const EPSILON: Self = Self::EPSILON;

    #[inline]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "rounding to the type is what the function does"
    )]
    fn of(value: f64) -> Self {
        value as Self
    }
    #[inline]
    #[expect(clippy::cast_precision_loss, reason = "weights tolerate rounding")]
    fn of_count(count: usize) -> Self {
        count as Self
    }
    #[inline]
    fn as_f64(self) -> f64 {
        f64::from(self)
    }
    #[inline]
    fn root(self) -> Self {
        libm::sqrtf(self)
    }
    #[inline]
    fn magnitude(self) -> Self {
        libm::fabsf(self)
    }
    #[inline]
    fn angle_over(self, along: Self) -> Self {
        libm::atan2f(self, along)
    }
    #[inline]
    fn finite(self) -> bool {
        self.is_finite()
    }
}

impl Real for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const INFINITY: Self = Self::INFINITY;
    const EPSILON: Self = Self::EPSILON;

    #[inline]
    fn of(value: f64) -> Self {
        value
    }
    #[inline]
    #[expect(clippy::cast_precision_loss, reason = "weights tolerate rounding")]
    fn of_count(count: usize) -> Self {
        count as Self
    }
    #[inline]
    fn as_f64(self) -> f64 {
        self
    }
    #[inline]
    fn root(self) -> Self {
        libm::sqrt(self)
    }
    #[inline]
    fn magnitude(self) -> Self {
        libm::fabs(self)
    }
    #[inline]
    fn angle_over(self, along: Self) -> Self {
        libm::atan2(self, along)
    }
    #[inline]
    fn finite(self) -> bool {
        self.is_finite()
    }
}
