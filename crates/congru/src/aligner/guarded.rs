use super::{Aligner, Estimate, Frame, start};
use crate::{Error, Icp, Multistart, Target};

/// A local [`Icp`] from the guess, and the [`Multistart`] search only when
/// the local result keeps less than `min_inliers` of the scan: the speed of
/// a local method when the guess is good, the reach of a global one when it
/// is not.
#[derive(Debug)]
pub struct Guarded {
    pub name: String,
    /// Should carry a correspondence distance, or every pair counts as kept.
    pub local: Icp,
    pub search: Multistart,
    pub min_inliers: f64,
    pub target: Target,
}

impl Aligner for Guarded {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        if let Ok(alignment) = self.local.align(frame.scan, &self.target, &start(frame))
            && alignment.inlier_fraction >= self.min_inliers
        {
            return Ok(Estimate::from_alignment(alignment));
        }
        Ok(Estimate::from_alignment(
            self.search.align(frame.scan, &self.target)?,
        ))
    }
}
