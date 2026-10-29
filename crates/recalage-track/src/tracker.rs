use recalage::{Alignment, Error, Icp, Multistart, PointCloud, RigidTransform};

use crate::{Gains, Health, ModelView, MotionFilter};

/// Which of the two phases produced a frame's pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Predicted from the last frames, refined by a few local iterations.
    Follow,
    /// Searched over all rotations: the first frame, or after a failure.
    Find,
}

/// One frame's outcome. `pose` maps the model's frame to the sensor's.
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub pose: RigidTransform,
    pub phase: Phase,
    /// Whether the next frame may follow from this one.
    pub trusted: bool,
    pub alignment: Alignment,
}

/// Follows one model through a stream of point clouds.
#[derive(Debug)]
pub struct Tracker<View> {
    pub view: View,
    /// Run every followed frame, from the prediction.
    pub follow: Icp,
    /// Run when there is nothing to follow from.
    pub find: Multistart,
    pub health: Health,
    filter: MotionFilter,
}

impl<View: ModelView> Tracker<View> {
    #[must_use]
    pub fn new(view: View, follow: Icp, find: Multistart, health: Health, gains: Gains) -> Self {
        Self {
            view,
            follow,
            find,
            health,
            filter: MotionFilter::new(gains),
        }
    }

    /// `scan` holds one frame's points, in the sensor's frame.
    ///
    /// # Errors
    /// When neither phase can align the frame; the next frame then starts
    /// with a search.
    pub fn update(&mut self, scan: &PointCloud) -> Result<Update, Error> {
        let (phase, outcome) = match self.filter.predict() {
            Some(predicted) => (Phase::Follow, self.follow_from(scan, &predicted)),
            None => (Phase::Find, self.find.align(scan, self.view.whole())),
        };
        let alignment = outcome.inspect_err(|_| self.filter.reset())?;
        let measured = alignment.transform.inverse();
        let trusted = self.health.trusts(&alignment);
        let pose = if trusted {
            let free = alignment
                .observability
                .as_ref()
                .map_or(&[][..], |report| &report.weak_motions[..]);
            self.filter.correct(&measured, free)
        } else {
            self.filter.reset();
            measured
        };
        Ok(Update {
            pose,
            phase,
            trusted,
            alignment,
        })
    }

    fn follow_from(
        &self,
        scan: &PointCloud,
        predicted: &RigidTransform,
    ) -> Result<Alignment, Error> {
        let seen = self.view.seen_from(predicted)?;
        let target = seen.as_ref().unwrap_or_else(|| self.view.whole());
        self.follow.align(scan, target, &predicted.inverse())
    }
}
