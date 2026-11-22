use super::correspond::{correspond, correspond_on, residual};
use crate::fit::{Match, kabsch, observe, point_to_plane};
use crate::geometry::count_as_float;
use crate::{
    Alignment, Error, Icp, Metric, PointCloud, RigidTransform, StopReason, Surface, Target,
};

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
        self.iterate(source, initial, |pose| {
            correspond(source, target, pose, self)
        })
    }

    /// As [`Icp::align`], onto a surface itself rather than samples of it: the
    /// nearest point is exact on a [`crate::MeshIndex`], a read on a
    /// [`crate::DistanceField`]. The metric still chooses what is minimised.
    ///
    /// # Errors
    /// [`Error::TooFewCorrespondences`] or [`Error::Degenerate`], as [`Icp::align`].
    pub fn align_on(
        &self,
        source: &PointCloud,
        surface: &impl Surface,
        initial: &RigidTransform,
    ) -> Result<Alignment, Error> {
        self.iterate(source, initial, |pose| {
            correspond_on(source, surface, pose, self)
        })
    }

    fn iterate(
        &self,
        source: &PointCloud,
        initial: &RigidTransform,
        pairs: impl Fn(&RigidTransform) -> Vec<Match>,
    ) -> Result<Alignment, Error> {
        let mut pose = *initial;
        for iteration in 1..=self.convergence.max_iterations {
            let deviation = initial.inverse().then(&pose);
            let step = self.solve(&pairs(&pose), &deviation)?;
            pose = pose.then(&step);
            if self.is_settled(&step) {
                return Self::finish(
                    source,
                    &pairs(&pose),
                    pose,
                    iteration,
                    StopReason::Converged,
                );
            }
        }
        Self::finish(
            source,
            &pairs(&pose),
            pose,
            self.convergence.max_iterations,
            StopReason::IterationLimit,
        )
    }

    fn solve(
        &self,
        matches: &[Match],
        deviation: &RigidTransform,
    ) -> Result<RigidTransform, Error> {
        match self.metric {
            Metric::PointToPoint => kabsch(matches),
            Metric::PointToPlane => {
                point_to_plane(matches, self.prior.as_ref().map(|prior| (prior, deviation)))
            }
        }
    }

    fn is_settled(&self, step: &RigidTransform) -> bool {
        step.rotation().angle() < self.convergence.rotation
            && step.translation().norm() < self.convergence.translation
    }

    fn finish(
        source: &PointCloud,
        matches: &[Match],
        pose: RigidTransform,
        iterations: u32,
        stop: StopReason,
    ) -> Result<Alignment, Error> {
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
            observability: observe(matches),
        })
    }
}
