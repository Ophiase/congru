use super::Vector3;
use crate::Real;

const SERIES_REACH: f64 = 0.25;
const MAX_HALVINGS: u32 = 64;

/// A Taylor series near zero, then the double-angle formulas. `sinf` of `libm`
/// computes in `f64`, which would pull soft doubles into every `f32` image.
fn sine_and_cosine<Float: Real>(angle: Float) -> (Float, Float) {
    let mut reduced = angle;
    let mut halvings = 0;
    while reduced.magnitude() > Float::of(SERIES_REACH) && halvings < MAX_HALVINGS {
        reduced = reduced * Float::of(0.5);
        halvings += 1;
    }
    let square = reduced * reduced;
    let (mut sine, mut cosine) = (Float::ONE, Float::ONE);
    for (sine_divisor, cosine_divisor) in [
        (Float::of(210.0), Float::of(182.0)),
        (Float::of(156.0), Float::of(132.0)),
        (Float::of(110.0), Float::of(90.0)),
        (Float::of(72.0), Float::of(56.0)),
        (Float::of(42.0), Float::of(30.0)),
        (Float::of(20.0), Float::of(12.0)),
        (Float::of(6.0), Float::of(2.0)),
    ] {
        sine = Float::ONE - square * sine / sine_divisor;
        cosine = Float::ONE - square * cosine / cosine_divisor;
    }
    sine = sine * reduced;
    for _ in 0..halvings {
        (sine, cosine) = (
            Float::of(2.0) * sine * cosine,
            cosine * cosine - sine * sine,
        );
    }
    (sine, cosine)
}

/// A rotation. Its norm is one to within rounding, restored at each
/// composition: in `f32` a matrix chained over iterations would drift, four
/// numbers renormalised do not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitQuaternion<Float> {
    real: Float,
    imaginary: Vector3<Float>,
}

impl<Float: Real> UnitQuaternion<Float> {
    #[must_use]
    pub fn identity() -> Self {
        Self {
            real: Float::ONE,
            imaginary: Vector3::ZERO,
        }
    }

    /// From any non-zero `[real, i, j, k]`; zero or not finite gives the
    /// identity.
    #[must_use]
    pub fn normalised(components: [Float; 4]) -> Self {
        let [real, first, second, third] = components;
        let imaginary = Vector3([first, second, third]);
        let norm = (real * real + imaginary.squared_norm()).root();
        if norm > Float::ZERO && norm.finite() {
            Self {
                real: real / norm,
                imaginary: imaginary * (Float::ONE / norm),
            }
        } else {
            Self::identity()
        }
    }

    /// A vector along the axis, its length the angle in radians. Accurate to
    /// a few `ε` up to a few turns.
    #[must_use]
    pub fn from_scaled_axis(axis: Vector3<Float>) -> Self {
        let half = axis.norm() * Float::of(0.5);
        if half == Float::ZERO {
            return Self::identity();
        }
        let (sine, cosine) = sine_and_cosine(half);
        let [first, second, third] = (axis * (Float::of(0.5) * sine / half)).0;
        Self::normalised([cosine, first, second, third])
    }

    /// `[real, i, j, k]`.
    #[must_use]
    pub fn components(self) -> [Float; 4] {
        let [first, second, third] = self.imaginary.0;
        [self.real, first, second, third]
    }

    /// This rotation first, then `next`.
    #[must_use]
    pub fn then(self, next: Self) -> Self {
        let imaginary = self.imaginary * next.real
            + next.imaginary * self.real
            + next.imaginary.cross(self.imaginary);
        let [first, second, third] = imaginary.0;
        Self::normalised([
            self.real * next.real - self.imaginary.dot(next.imaginary),
            first,
            second,
            third,
        ])
    }

    #[must_use]
    pub fn inverse(self) -> Self {
        Self {
            real: self.real,
            imaginary: -self.imaginary,
        }
    }

    /// In radians, within `[0, π]`.
    #[must_use]
    pub fn angle(self) -> Float {
        Float::of(2.0) * self.imaginary.norm().angle_over(self.real.magnitude())
    }

    /// By rows, for applying to many points.
    #[must_use]
    pub fn matrix(self) -> [[Float; 3]; 3] {
        let one = Float::ONE;
        let two = Float::of(2.0);
        let [first, second, third] = self.imaginary.0;
        let real = self.real;
        [
            [
                one - two * (second * second + third * third),
                two * (first * second - real * third),
                two * (first * third + real * second),
            ],
            [
                two * (first * second + real * third),
                one - two * (first * first + third * third),
                two * (second * third - real * first),
            ],
            [
                two * (first * third - real * second),
                two * (second * third + real * first),
                one - two * (first * first + second * second),
            ],
        ]
    }
}
