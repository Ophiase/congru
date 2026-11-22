use congru::{Error, Fraction};
use congru_synth::{Corruption, Outliers, Resampling};

/// One situation of the registration suite: what the sensor does to the
/// model, and how far the starting guess is from the truth. Every distance
/// is a share of the model's size, so one situation fits every model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Situation {
    pub name: &'static str,
    /// What the situation stresses; the recommendations are given per class.
    pub class: &'static str,
    /// Standard deviation of the noise, as a share of the model's size.
    pub noise: f64,
    /// Outliers, as a share of the observed points.
    pub outliers: f64,
    /// Share of the model the sensor sees, cut by a random plane.
    pub visible: f64,
    /// Share of the observed samples kept.
    pub density: f64,
    /// Rotation of the starting guess away from the truth, in degrees.
    pub turn: f64,
    /// Shift of the starting guess, as a share of the model's size.
    pub shift: f64,
}

const NOMINAL: Situation = Situation {
    name: "nominal",
    class: "nominal",
    noise: 0.005,
    outliers: 0.0,
    visible: 1.0,
    density: 1.0,
    turn: 10.0,
    shift: 0.02,
};

/// The guess a detector gives in a realistic scene: 10° and 3 % off.
pub const DETECTOR: Situation = Situation {
    name: "detector guess",
    class: "realistic",
    turn: 10.0,
    shift: 0.03,
    ..NOMINAL
};

/// One trouble at a time on top of the nominal situation.
pub const SITUATIONS: [Situation; 13] = [
    NOMINAL,
    Situation {
        name: "noise 2 %",
        class: "noise",
        noise: 0.02,
        ..NOMINAL
    },
    Situation {
        name: "noise 5 %",
        class: "noise",
        noise: 0.05,
        ..NOMINAL
    },
    Situation {
        name: "outliers 30 %",
        class: "outliers",
        outliers: 0.3,
        ..NOMINAL
    },
    Situation {
        name: "outliers 100 %",
        class: "outliers",
        outliers: 1.0,
        ..NOMINAL
    },
    Situation {
        name: "outliers 300 %",
        class: "outliers",
        outliers: 3.0,
        ..NOMINAL
    },
    Situation {
        name: "half visible",
        class: "partial view",
        visible: 0.5,
        ..NOMINAL
    },
    Situation {
        name: "quarter visible",
        class: "partial view",
        visible: 0.25,
        ..NOMINAL
    },
    Situation {
        name: "sparse",
        class: "sparse",
        density: 0.05,
        ..NOMINAL
    },
    Situation {
        name: "guess 30°",
        class: "far guess",
        turn: 30.0,
        shift: 0.05,
        ..NOMINAL
    },
    Situation {
        name: "guess 60°",
        class: "far guess",
        turn: 60.0,
        shift: 0.1,
        ..NOMINAL
    },
    Situation {
        name: "no guess",
        class: "no guess",
        turn: 180.0,
        shift: 0.3,
        ..NOMINAL
    },
    Situation {
        name: "everything",
        class: "combined",
        noise: 0.02,
        outliers: 0.3,
        visible: 0.5,
        density: 0.3,
        turn: 20.0,
        shift: 0.05,
    },
];

impl Situation {
    /// The corruption for a model of size `span` observed as `count` points.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] for a visible share outside `(0, 1]`.
    pub fn corruption(&self, span: f64, count: usize) -> Result<Corruption, Error> {
        Ok(Corruption {
            resampling: Resampling::One,
            noise: self.noise * span,
            outliers: Outliers::InBox {
                count: crate::measure::share_of(count, self.outliers),
            },
            visible: Fraction::new(self.visible)?,
        })
    }
}
