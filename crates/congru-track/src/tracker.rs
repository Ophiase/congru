use congru::{
    Alignment, Distance, Error, Icp, Kernel, Multistart, PointCloud, Prior, RigidTransform,
};

use crate::noise::{LearnedNoise, deviation_of};
use crate::{Gains, Health, ModelView, MotionFilter};

/// Tukey's constant for 95 % efficiency on Gaussian residuals.
const KERNEL_DEVIATIONS: f64 = 4.685;
const GATE_DEVIATIONS: f64 = 5.0;

/// Which of the two phases produced a frame's pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Predicted from the last frames, refined by local iterations pulled
    /// toward the prediction.
    Follow,
    /// Searched over all rotations: the first frame, or a frame the follow
    /// phase could not explain.
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

/// How far the part may plausibly move between two frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Agility {
    /// In radians.
    pub rotation: f64,
    pub translation: f64,
}

/// Follows one model through a stream of point clouds.
#[derive(Debug)]
pub struct Tracker<View> {
    pub view: View,
    /// The local refinement of a followed frame; its kernel, gate and prior
    /// are set each frame from the learned noise.
    pub follow: Icp,
    pub find: Multistart,
    pub health: Health,
    pub agility: Agility,
    filter: MotionFilter,
    noise: LearnedNoise,
}

impl<View: ModelView> Tracker<View> {
    #[must_use]
    pub fn new(
        view: View,
        follow: Icp,
        find: Multistart,
        health: Health,
        agility: Agility,
        gains: Gains,
        noise_floor: f64,
    ) -> Self {
        Self {
            view,
            follow,
            find,
            health,
            agility,
            filter: MotionFilter::new(gains),
            noise: LearnedNoise::new(noise_floor),
        }
    }

    /// `scan` holds one frame's points, in the sensor's frame. A frame the
    /// follow phase cannot explain is searched again at once.
    ///
    /// # Errors
    /// When the search fails too; the next frame starts with a search.
    pub fn update(&mut self, scan: &PointCloud) -> Result<Update, Error> {
        if let Some(predicted) = self.filter.predict()
            && let Ok((alignment, deviation)) = self.follow_from(scan, &predicted)
            && self
                .health
                .trusts(alignment.inlier_fraction, deviation, self.noise.deviation())
        {
            return Ok(self.accept(alignment, deviation, Phase::Follow));
        }
        self.filter.reset();
        let alignment = self.find.align(scan, self.view.whole())?;
        let deviation = deviation_of(scan, self.view.whole(), &alignment.transform);
        let lenient = self.noise.deviation().map(|known| known * 2.0);
        if self
            .health
            .trusts(alignment.inlier_fraction, deviation, lenient)
        {
            return Ok(self.accept(alignment, deviation, Phase::Find));
        }
        Ok(Update {
            pose: alignment.transform.inverse(),
            phase: Phase::Find,
            trusted: false,
            alignment,
        })
    }

    fn accept(&mut self, alignment: Alignment, deviation: f64, phase: Phase) -> Update {
        self.noise.learn(deviation);
        let free = alignment
            .observability
            .as_ref()
            .map_or(&[][..], |report| &report.weak_motions[..]);
        let pose = self.filter.correct(&alignment.transform.inverse(), free);
        Update {
            pose,
            phase,
            trusted: true,
            alignment,
        }
    }

    fn follow_from(
        &self,
        scan: &PointCloud,
        predicted: &RigidTransform,
    ) -> Result<(Alignment, f64), Error> {
        let seen = self.view.seen_from(predicted)?;
        let target = seen.as_ref().unwrap_or_else(|| self.view.whole());
        let alignment = self.tuned()?.align(scan, target, &predicted.inverse())?;
        let deviation = deviation_of(scan, target, &alignment.transform);
        Ok((alignment, deviation))
    }

    fn tuned(&self) -> Result<Icp, Error> {
        let Some(deviation) = self.noise.deviation() else {
            return Ok(self.follow);
        };
        Ok(Icp {
            kernel: Kernel::Tukey(Distance::new(KERNEL_DEVIATIONS * deviation)?),
            max_correspondence_distance: Some(Distance::new(GATE_DEVIATIONS * deviation)?),
            prior: Some(Prior::new(
                self.agility.rotation,
                self.agility.translation,
                deviation,
            )?),
            ..self.follow
        })
    }
}
