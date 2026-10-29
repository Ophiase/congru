use congru::{Aligner, Error, Icp, Target};
use congru_track::{SensorView, Settings, Tracker, WholeModel};

use super::Model;
use super::builders::{
    adaptive, annealed, consensus, guarded, local, point, search, trimmed, tukey,
};

/// Whether a method answers one frame alone, or follows a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    OneShot,
    Tracking,
}

/// A method of the benchmark: how to build it for a model.
#[derive(Debug, Clone, Copy)]
pub struct Method {
    pub name: &'static str,
    pub family: Family,
    /// Also run frame by frame on the sequences, as a baseline for the
    /// trackers.
    pub follows: bool,
    pub summary: &'static str,
    pub build: fn(&Model) -> Result<Box<dyn Aligner>, Error>,
}

/// Every method compared. Adding one is adding a line here.
pub const METHODS: [Method; 12] = [
    one_shot("icp-point", "point-to-point ICP, least squares", |model| {
        local(model, point())
    }),
    one_shot("icp-plane", "point-to-plane ICP, least squares", |model| {
        local(model, Icp::default())
    }),
    baseline(
        "icp-tukey",
        "point-to-plane ICP, Tukey kernel, gated",
        |model| local(model, tukey(model.span)?),
    ),
    one_shot(
        "icp-trimmed",
        "point-to-plane ICP, Cauchy kernel, 70 % trimmed",
        |model| local(model, trimmed(model.span)?),
    ),
    one_shot(
        "annealed",
        "point-to-plane ICP, Cauchy kernel annealed from coarse to fine",
        annealed,
    ),
    baseline(
        "adaptive",
        "point-to-plane ICP, Tukey kernel and gate set each stage from the median distance",
        adaptive,
    ),
    one_shot(
        "search",
        "multistart over 32 rotations, Cauchy kernel",
        search,
    ),
    one_shot(
        "consensus",
        "FPFH descriptors, mutual matches, RANSAC, checked on the whole shape, refined by Tukey ICP",
        consensus,
    ),
    baseline(
        "guarded",
        "Tukey ICP, multistart only when it keeps under 80 %",
        guarded,
    ),
    tracking(
        "tracker",
        "two-phase tracker on the side facing the sensor",
        |model| {
            Ok(Box::new(Tracker::new(
                "tracker",
                SensorView::new(model.target.clone())?,
                Settings::for_part(model.span)?,
            )))
        },
    ),
    tracking(
        "tracker-whole",
        "two-phase tracker on the whole model",
        |model| {
            let view = WholeModel(Target::from_oriented(model.target.clone())?);
            Ok(Box::new(Tracker::new(
                "tracker-whole",
                view,
                Settings::for_part(model.span)?,
            )))
        },
    ),
    tracking(
        "tracker-loose",
        "two-phase tracker, motion prior made negligible",
        |model| {
            let mut settings = Settings::for_part(model.span)?;
            settings.agility.rotation = 10.0;
            settings.agility.translation = 10.0 * model.span;
            Ok(Box::new(Tracker::new(
                "tracker-loose",
                SensorView::new(model.target.clone())?,
                settings,
            )))
        },
    ),
];

const fn one_shot(
    name: &'static str,
    summary: &'static str,
    build: fn(&Model) -> Result<Box<dyn Aligner>, Error>,
) -> Method {
    Method {
        name,
        family: Family::OneShot,
        follows: false,
        summary,
        build,
    }
}

const fn baseline(
    name: &'static str,
    summary: &'static str,
    build: fn(&Model) -> Result<Box<dyn Aligner>, Error>,
) -> Method {
    Method {
        name,
        family: Family::OneShot,
        follows: true,
        summary,
        build,
    }
}

const fn tracking(
    name: &'static str,
    summary: &'static str,
    build: fn(&Model) -> Result<Box<dyn Aligner>, Error>,
) -> Method {
    Method {
        name,
        family: Family::Tracking,
        follows: true,
        summary,
        build,
    }
}
