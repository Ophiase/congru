use congru::{
    Adaptive, Aligner, Annealed, Consensus, Distance, Error, Guarded, Icp, Kernel, Local,
    Multistart, Search, Target,
};

use crate::model::Model;

/// The alignment methods the command offers, every scale set from the
/// model's size as the benchmark sets them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Method {
    Local,
    Search,
    Annealed,
    Adaptive,
    Guarded,
    Consensus,
}

impl Method {
    pub const ALL: [Self; 6] = [
        Self::Local,
        Self::Search,
        Self::Annealed,
        Self::Adaptive,
        Self::Guarded,
        Self::Consensus,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Search => "search",
            Self::Annealed => "annealed",
            Self::Adaptive => "adaptive",
            Self::Guarded => "guarded",
            Self::Consensus => "consensus",
        }
    }

    #[must_use]
    pub const fn summary(self) -> &'static str {
        match self {
            Self::Local => {
                "point-to-plane ICP from the guess, Tukey kernel, gated: fast, needs a guess within about 20 degrees"
            }
            Self::Search => {
                "ICP from 32 rotations spread over all orientations, the best kept: needs no guess"
            }
            Self::Annealed => {
                "ICP from the guess, Cauchy kernel shrinking from coarse to fine: a wider basin than local"
            }
            Self::Adaptive => {
                "ICP from the guess, kernel and gate set each stage from the measured noise: no scale to tune"
            }
            Self::Guarded => {
                "local ICP, then the search only when it keeps under 80 % of the scan: fast with a good guess, global without"
            }
            Self::Consensus => {
                "shape descriptors matched and agreed on by RANSAC, refined by ICP: needs no guess, copes with a partial view"
            }
        }
    }

    /// Whether the method starts from `--guess`; the others ignore it.
    #[must_use]
    pub const fn uses_guess(self) -> bool {
        matches!(
            self,
            Self::Local | Self::Annealed | Self::Adaptive | Self::Guarded
        )
    }

    /// # Errors
    /// When the model is too small or flat for the method's scales.
    pub fn build(self, model: &Model) -> Result<Box<dyn Aligner>, Error> {
        let span = model.span;
        let name = self.name().to_owned();
        let target = || Target::from_oriented(model.surface.clone());
        Ok(match self {
            Self::Local => Box::new(Local {
                name,
                icp: tukey(span)?,
                target: target()?,
            }),
            Self::Search => Box::new(Search {
                name,
                multistart: multistart(span)?,
                target: target()?,
            }),
            Self::Guarded => Box::new(Guarded {
                name,
                local: tukey(span)?,
                search: multistart(span)?,
                min_inliers: 0.8,
                target: target()?,
            }),
            Self::Annealed => Box::new(Annealed {
                name,
                icp: Icp::default(),
                coarsest: Distance::new(span / 5.0)?,
                finest: Distance::new(span / 80.0)?,
                target: target()?,
            }),
            Self::Adaptive => adaptive(name, span, target()?)?,
            Self::Consensus => Box::new(Consensus::new(
                name,
                &model.surface,
                Distance::new(span / 30.0)?,
                48,
                tukey(span)?,
            )?),
        })
    }
}

fn adaptive(name: String, span: f64, target: Target) -> Result<Box<dyn Aligner>, Error> {
    Ok(Box::new(Adaptive {
        name,
        icp: Icp::default(),
        first_deviation: Distance::new(span / 20.0)?,
        stages: 4,
        target,
    }))
}

fn tukey(span: f64) -> Result<Icp, Error> {
    Ok(Icp {
        kernel: Kernel::Tukey(Distance::new(span / 30.0)?),
        max_correspondence_distance: Some(Distance::new(span / 10.0)?),
        ..Icp::default()
    })
}

fn multistart(span: f64) -> Result<Multistart, Error> {
    Ok(Multistart {
        starts: 32,
        icp: Icp {
            kernel: Kernel::Cauchy(Distance::new(span / 40.0)?),
            ..Icp::default()
        },
    })
}
