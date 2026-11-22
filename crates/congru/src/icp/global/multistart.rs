use super::rotations::spread_rotations;
use super::score::{default_threshold, truncated_cost};
use crate::search::map;
use crate::{Alignment, Error, Icp, PointCloud, RigidTransform, Target};

/// Alignment without an initial pose: the centroids are matched, ICP runs
/// from `starts` rotations spread evenly over SO(3), and the result
/// explaining the most of the source wins.
///
/// Fits a source covering most of the target. A small partial view moves
/// the centroid and calls for an initial pose and [`Icp`] instead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multistart {
    /// How many rotations to start from; 32 to 128 for an asymmetric shape.
    pub starts: usize,
    pub icp: Icp,
}

impl Default for Multistart {
    fn default() -> Self {
        Self {
            starts: 64,
            icp: Icp::default(),
        }
    }
}

impl Multistart {
    /// # Errors
    /// [`Error::InvalidParameter`] for zero starts; otherwise the error of
    /// the first start when every start fails.
    pub fn align(&self, source: &PointCloud, target: &Target) -> Result<Alignment, Error> {
        let threshold = self
            .icp
            .max_correspondence_distance
            .map_or_else(|| default_threshold(target), crate::Distance::get);
        let initials = self.initial_poses(source, target)?;
        let outcomes = map(&initials, |initial| {
            self.icp.align(source, target, initial).map(|alignment| {
                let cost = truncated_cost(source, target, &alignment.transform, threshold);
                (cost, alignment)
            })
        });
        let (successes, failures): (Vec<_>, Vec<_>) = outcomes.into_iter().partition(Result::is_ok);
        successes
            .into_iter()
            .flatten()
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, alignment)| alignment)
            .ok_or_else(|| {
                failures
                    .into_iter()
                    .find_map(Result::err)
                    .unwrap_or(Error::Degenerate)
            })
    }

    fn initial_poses(
        &self,
        source: &PointCloud,
        target: &Target,
    ) -> Result<Vec<RigidTransform>, Error> {
        if self.starts == 0 {
            return Err(Error::InvalidParameter {
                name: "starts",
                expected: "at least 1",
                value: 0.0,
            });
        }
        let source_centre = source.centroid().coords;
        let target_centre = target.cloud().centroid().coords;
        Ok(spread_rotations(self.starts)
            .into_iter()
            .map(|rotation| {
                RigidTransform::from_parts(rotation, target_centre - rotation * source_centre)
            })
            .collect())
    }
}
