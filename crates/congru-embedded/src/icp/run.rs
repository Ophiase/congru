use super::workspace::{Match, NO_MATCH};
use super::{Alignment, Icp, Metric, StopReason, Workspace};
use crate::fit::point_to_plane;
use crate::linear::Frozen;
use crate::{Error, KdTree, Pair, Pose, Real, Vector3, kabsch};

impl<Float: Real> Icp<Float> {
    /// Moves `source` onto `target` from `initial`. The answer takes a source
    /// point to the target's frame. Allocates nothing.
    ///
    /// # Errors
    /// [`Error::Empty`] for an empty source, [`Error::WorkspaceTooSmall`],
    /// [`Error::MissingNormals`] for point-to-plane without target normals,
    /// [`Error::InvalidParameter`] for a gate or a kernel scale not positive
    /// and finite; [`Error::TooFewPairs`] or [`Error::Degenerate`] when an
    /// iteration keeps too few or too poorly spread pairs, usually a bad
    /// initial pose or a gate too tight.
    pub fn align(
        &self,
        source: &[[Float; 3]],
        target: &KdTree<Float>,
        initial: &Pose<Float>,
        workspace: &mut Workspace<Float>,
    ) -> Result<Alignment<Float>, Error> {
        self.check(source, target, workspace)?;
        let matches = &mut workspace.matches[..source.len()];
        let mut pose = *initial;
        for iteration in 1..=self.convergence.max_iterations {
            let frozen = pose.frozen();
            self.correspond(source, target, &frozen, matches);
            let step = self.solve(source, target, &frozen, matches)?;
            pose = pose.then(&step);
            if self.is_settled(&step) {
                let reason = StopReason::Converged;
                return Ok(self.finish(source, target, pose, iteration, reason, matches));
            }
        }
        let budget = self.convergence.max_iterations;
        let reason = StopReason::IterationLimit;
        Ok(self.finish(source, target, pose, budget, reason, matches))
    }

    fn check(
        &self,
        source: &[[Float; 3]],
        target: &KdTree<Float>,
        workspace: &Workspace<Float>,
    ) -> Result<(), Error> {
        if source.is_empty() {
            return Err(Error::Empty);
        }
        if workspace.matches.len() < source.len() {
            return Err(Error::WorkspaceTooSmall {
                capacity: workspace.matches.len(),
                needed: source.len(),
            });
        }
        if self.metric == Metric::PointToPlane && target.normals().is_none() {
            return Err(Error::MissingNormals);
        }
        if self.gate.is_some_and(|gate| !gate.strictly_positive()) {
            return Err(Error::InvalidParameter { name: "gate" });
        }
        if !self.kernel.scale_is_valid() {
            return Err(Error::InvalidParameter {
                name: "kernel scale",
            });
        }
        Ok(())
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tree refuses 2^32 points at its making"
    )]
    fn correspond(
        &self,
        source: &[[Float; 3]],
        target: &KdTree<Float>,
        pose: &Frozen<Float>,
        matches: &mut [Match<Float>],
    ) {
        let limit = self.gate.map_or(Float::INFINITY, |gate| gate * gate);
        for (point, slot) in source.iter().zip(matches) {
            let moved = pose.apply(Vector3(*point));
            *slot = target.nearest_within(moved, limit).map_or(
                Match {
                    target: NO_MATCH,
                    weight: Float::ZERO,
                },
                |nearest| Match {
                    target: nearest.index as u32,
                    weight: self.kernel.weight(self.residual(
                        target,
                        moved,
                        nearest.index,
                        nearest.squared_distance,
                    )),
                },
            );
        }
    }

    fn residual(
        &self,
        target: &KdTree<Float>,
        moved: Vector3<Float>,
        index: usize,
        squared_distance: Float,
    ) -> Float {
        match (self.metric, target.normals()) {
            (Metric::PointToPlane, Some(normals)) => {
                normals[index].dot(moved - target.points()[index])
            }
            _ => squared_distance.root(),
        }
    }

    fn solve(
        &self,
        source: &[[Float; 3]],
        target: &KdTree<Float>,
        pose: &Frozen<Float>,
        matches: &[Match<Float>],
    ) -> Result<Pose<Float>, Error> {
        let pairs = source
            .iter()
            .zip(matches)
            .filter(|(_, slot)| slot.target != NO_MATCH && slot.weight > Float::ZERO)
            .map(|(point, slot)| Pair {
                source: pose.apply(Vector3(*point)),
                target: target.points()[slot.target as usize],
                weight: slot.weight,
            });
        match (self.metric, target.normals()) {
            (Metric::PointToPlane, Some(normals)) => {
                let indices = matches
                    .iter()
                    .filter(|slot| slot.target != NO_MATCH && slot.weight > Float::ZERO);
                point_to_plane(
                    pairs
                        .zip(indices)
                        .map(|(pair, slot)| (pair, normals[slot.target as usize])),
                )
            }
            _ => kabsch(pairs),
        }
    }

    fn is_settled(&self, step: &Pose<Float>) -> bool {
        step.rotation.angle() < self.convergence.rotation
            && step.translation.norm() < self.convergence.translation
    }

    fn finish(
        &self,
        source: &[[Float; 3]],
        target: &KdTree<Float>,
        pose: Pose<Float>,
        iterations: u32,
        stop: StopReason,
        matches: &mut [Match<Float>],
    ) -> Alignment<Float> {
        let frozen = pose.frozen();
        self.correspond(source, target, &frozen, matches);
        let mut kept = 0;
        let mut squared = Float::ZERO;
        for (point, slot) in source.iter().zip(matches.iter()) {
            if slot.target != NO_MATCH {
                let moved = frozen.apply(Vector3(*point));
                kept += 1;
                squared += (moved - target.points()[slot.target as usize]).squared_norm();
            }
        }
        let rmse = if kept == 0 {
            Float::ZERO
        } else {
            (squared / Float::of_count(kept)).root()
        };
        Alignment {
            pose,
            rmse,
            inlier_fraction: Float::of_count(kept) / Float::of_count(source.len()),
            iterations,
            stop,
        }
    }
}
