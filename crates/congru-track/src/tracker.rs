use congru::{Alignment, Distance, Error, Icp, Kernel, PointCloud, Prior, RigidTransform};

use crate::noise::{LearnedNoise, deviation_of};
use crate::{ModelView, MotionFilter, Settings};

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

impl Update {
    /// What the frame alone says of the pose, before the tracker's filter: `pose`
    /// is steadier and lags behind what a jolt of the camera did at once.
    #[must_use]
    pub fn measured(&self) -> RigidTransform {
        self.alignment.transform.inverse()
    }
}

/// Follows one model through a stream of point clouds.
#[derive(Debug)]
pub struct Tracker<View> {
    pub name: String,
    pub view: View,
    pub settings: Settings,
    filter: MotionFilter,
    noise: LearnedNoise,
}

impl<View: ModelView> Tracker<View> {
    #[must_use]
    pub fn new(name: impl Into<String>, view: View, settings: Settings) -> Self {
        Self {
            name: name.into(),
            view,
            settings,
            filter: MotionFilter::new(settings.gains),
            noise: LearnedNoise::new(settings.noise_floor),
        }
    }

    /// `scan` holds one frame's points, in the sensor's frame. A frame the
    /// follow phase cannot explain is searched again at once.
    ///
    /// # Errors
    /// When the search fails too; the next frame starts with a search.
    pub fn update(&mut self, scan: &PointCloud) -> Result<Update, Error> {
        self.update_from(scan, None)
    }

    /// As [`Tracker::update`], following from `guess` when there is nothing
    /// to predict from yet.
    ///
    /// # Errors
    /// As [`Tracker::update`].
    pub fn update_from(
        &mut self,
        scan: &PointCloud,
        guess: Option<RigidTransform>,
    ) -> Result<Update, Error> {
        if let Some(predicted) = self.filter.predict().or(guess)
            && let Ok((alignment, deviation)) = self.follow_from(scan, &predicted)
            && self.settings.health.trusts(
                alignment.inlier_fraction,
                deviation,
                self.noise.deviation(),
            )
        {
            return Ok(self.accept(alignment, deviation, Phase::Follow));
        }
        self.filter.reset();
        let alignment = self.settings.find.align(scan, self.view.whole())?;
        let deviation = deviation_of(scan, self.view.whole(), &alignment.transform);
        let lenient = self.noise.deviation().map(|known| known * 2.0);
        if self
            .settings
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

    /// `scan` aligned from `start`, already close, through a short gate.
    ///
    /// # Errors
    /// When `start` shows nothing of the model, or too few points lie in the gate.
    pub fn aligned_near(
        &self,
        scan: &PointCloud,
        start: &RigidTransform,
    ) -> Result<RigidTransform, Error> {
        let seen = self.view.seen_from(start)?;
        let target = seen.as_ref().unwrap_or_else(|| self.view.whole());
        let gate = Distance::new(self.settings.near_gate)?;
        let icp = Icp {
            kernel: Kernel::Tukey(gate),
            max_correspondence_distance: Some(gate),
            ..self.settings.follow
        };
        Ok(icp
            .align(scan, target, &start.inverse())?
            .transform
            .inverse())
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
            return Ok(self.settings.follow);
        };
        Ok(Icp {
            kernel: Kernel::Tukey(Distance::new(KERNEL_DEVIATIONS * deviation)?),
            max_correspondence_distance: Some(Distance::new(GATE_DEVIATIONS * deviation)?),
            prior: Some(Prior::new(
                self.settings.agility.rotation,
                self.settings.agility.translation,
                deviation,
            )?),
            ..self.settings.follow
        })
    }
}

impl<View: ModelView> congru::Aligner for Tracker<View> {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &congru::Frame<'_>) -> Result<congru::Estimate, Error> {
        let update = self.update_from(frame.scan, frame.guess)?;
        Ok(congru::Estimate {
            pose: update.pose,
            trusted: update.trusted,
            observability: update.alignment.observability,
        })
    }

    fn reset(&mut self) {
        self.filter.reset();
        self.noise = LearnedNoise::new(self.settings.noise_floor);
    }
}
