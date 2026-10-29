use super::correspond::{correspond, residual};
use crate::fit::{Match, kabsch, point_to_plane};
use crate::geometry::count_as_float;
use crate::{Alignment, Error, Icp, Metric, PointCloud, RigidTransform, StopReason, Target};

impl Icp {
    /// # Errors
    /// [`Error::MissingNormals`] for point-to-plane on a target without
    /// normals; [`Error::TooFewCorrespondences`] or [`Error::Degenerate`]
    /// when an iteration keeps too few or too poorly spread pairs, usually a
    /// sign of a bad initial pose or a correspondence distance too tight.
    pub fn align(
        &self,
        source: &PointCloud,
        target: &Target,
        initial: &RigidTransform,
    ) -> Result<Alignment, Error> {
        if self.metric == Metric::PointToPlane && target.normals().is_none() {
            return Err(Error::MissingNormals);
        }
        let mut pose = *initial;
        for iteration in 1..=self.convergence.max_iterations {
            let step = self.solve(&correspond(source, target, &pose, self))?;
            pose = pose.then(&step);
            if self.is_settled(&step) {
                return self.finish(source, target, pose, iteration, StopReason::Converged);
            }
        }
        self.finish(
            source,
            target,
            pose,
            self.convergence.max_iterations,
            StopReason::IterationLimit,
        )
    }

    fn solve(&self, matches: &[Match]) -> Result<RigidTransform, Error> {
        match self.metric {
            Metric::PointToPoint => kabsch(matches),
            Metric::PointToPlane => point_to_plane(matches),
        }
    }

    fn is_settled(&self, step: &RigidTransform) -> bool {
        step.rotation().angle() < self.convergence.rotation
            && step.translation().norm() < self.convergence.translation
    }

    fn finish(
        &self,
        source: &PointCloud,
        target: &Target,
        pose: RigidTransform,
        iterations: u32,
        stop: StopReason,
    ) -> Result<Alignment, Error> {
        let matches = correspond(source, target, &pose, self);
        if matches.is_empty() {
            return Err(Error::TooFewCorrespondences {
                found: 0,
                needed: 1,
            });
        }
        let squared: f64 = matches
            .iter()
            .map(|pair| residual(pair, Metric::PointToPoint).powi(2))
            .sum();
        Ok(Alignment {
            transform: pose,
            rmse: (squared / count_as_float(matches.len())).sqrt(),
            inlier_fraction: count_as_float(matches.len()) / count_as_float(source.count()),
            iterations,
            stop,
        })
    }
}
