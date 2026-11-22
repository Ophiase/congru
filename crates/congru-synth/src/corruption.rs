use congru::{Distance, Fraction};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resampling {
    One,
    Fixed(u32),
    /// A Poisson count of the given mean: some points vanish, some repeat.
    Poisson(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outliers {
    None,
    /// Uniform in the model's bounding box, grown by a tenth on each side.
    InBox {
        count: usize,
    },
    /// Uniform in the grown bounding box, but never within `clearance` of
    /// the model: noise that cannot be mistaken for the surface.
    Exterior {
        count: usize,
        clearance: Distance,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corruption {
    pub resampling: Resampling,
    /// Standard deviation of the isotropic Gaussian noise of each observation.
    pub noise: f64,
    pub outliers: Outliers,
    /// Share of the model kept, cut by a random plane: a partial view.
    pub visible: Fraction,
}

impl Default for Corruption {
    fn default() -> Self {
        Self {
            resampling: Resampling::One,
            noise: 0.0,
            outliers: Outliers::None,
            visible: Fraction::ALL,
        }
    }
}
