use congru::{Distance, Error, Icp, Kernel, Multistart};

use crate::Gains;

/// When a frame's alignment is trusted enough to follow from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    /// Smallest share of the frame's points that must find the model within
    /// the gate: an occluder or clutter lowers it a little, a wrong pose a
    /// lot.
    pub min_inliers: f64,
    /// Largest noise of a frame, as a multiple of the noise learned so far.
    pub max_noise_ratio: f64,
}

impl Health {
    #[must_use]
    pub fn trusts(&self, inliers: f64, deviation: f64, learned: Option<f64>) -> bool {
        inliers >= self.min_inliers
            && learned.is_none_or(|known| deviation <= self.max_noise_ratio * known)
    }
}

/// How far the part may plausibly move between two frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Agility {
    /// In radians.
    pub rotation: f64,
    pub translation: f64,
}

/// Everything a tracker is tuned by. [`Settings::for_part`] holds the
/// defaults the benchmark chose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The local refinement of a followed frame; its kernel, gate and prior
    /// are replaced each frame once the noise is learned.
    pub follow: Icp,
    pub find: Multistart,
    pub health: Health,
    pub agility: Agility,
    pub gains: Gains,
    /// The learned noise never goes below this: the model's own sampling
    /// spacing is a good value.
    pub noise_floor: f64,
}

impl Settings {
    /// Defaults for a part of size `span`, in the unit of the points.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] for a span that is not finite and
    /// positive.
    pub fn for_part(span: f64) -> Result<Self, Error> {
        let kernel = Kernel::Cauchy(Distance::new(span / 20.0)?);
        Ok(Self {
            follow: Icp {
                kernel,
                max_correspondence_distance: Some(Distance::new(span / 7.0)?),
                convergence: congru::Convergence {
                    max_iterations: 20,
                    ..congru::Convergence::default()
                },
                ..Icp::default()
            },
            find: Multistart {
                starts: 64,
                icp: Icp {
                    kernel,
                    ..Icp::default()
                },
            },
            health: Health {
                min_inliers: 0.5,
                max_noise_ratio: 2.0,
            },
            agility: Agility {
                rotation: 0.08,
                translation: span / 20.0,
            },
            gains: Gains {
                pose: 0.6,
                velocity: 0.2,
            },
            noise_floor: span / 100.0,
        })
    }
}
