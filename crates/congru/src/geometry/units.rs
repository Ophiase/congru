use crate::Error;

/// A strictly positive, finite length, in the unit of the point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance(f64);

impl Distance {
    /// # Errors
    /// [`Error::InvalidParameter`] unless `value` is finite and positive.
    pub fn new(value: f64) -> Result<Self, Error> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(Error::InvalidParameter {
                name: "distance",
                expected: "finite and positive",
                value,
            })
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

/// A proportion in `(0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Fraction(f64);

impl Fraction {
    pub const ALL: Self = Self(1.0);

    /// # Errors
    /// [`Error::InvalidParameter`] unless `0 < value <= 1`.
    pub fn new(value: f64) -> Result<Self, Error> {
        if value > 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(Error::InvalidParameter {
                name: "fraction",
                expected: "in (0, 1]",
                value,
            })
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl Default for Fraction {
    fn default() -> Self {
        Self::ALL
    }
}
