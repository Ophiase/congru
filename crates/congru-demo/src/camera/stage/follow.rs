use congru::{Aligner, Distance, PointCloud, RigidTransform};
use congru_track::{Settings, Tracker, Update};
use congru_vision::DepthImage;

use super::super::detect::Detector;
use super::super::view::CameraView;
use super::hold::Hold;
use super::{MODEL_SPACING, PART_SPAN, Stage};

/// Everything one run remembers from a frame to the next.
#[derive(Debug)]
pub(super) struct Follower {
    tracker: Tracker<CameraView>,
    pub(super) detector: Detector,
    hold: Hold,
}

impl Follower {
    pub(super) fn new(stage: &Stage) -> anyhow::Result<Self> {
        let view = CameraView::new(&stage.part, stage.camera, Distance::new(MODEL_SPACING)?)?;
        let settings = Settings::for_part(PART_SPAN)?;
        Ok(Self {
            tracker: Tracker::new("tracker", view, settings),
            detector: Detector::new(&stage.part, PART_SPAN)?,
            hold: Hold::new(settings.motion),
        })
    }

    pub(super) fn forecast(&self) -> Option<RigidTransform> {
        self.hold.forecast()
    }

    /// One frame from the pose it starts `from`: the scan, and the answer. In a
    /// crowded frame the rays have the say, and the tracker, which only clutter
    /// would feed, starts over when the frame clears.
    pub(super) fn step(
        &mut self,
        stage: &Stage,
        (depth, region): (&DepthImage, &DepthImage),
        from: Option<RigidTransform>,
    ) -> (Option<PointCloud>, Answer) {
        let scan = from.and_then(|start| stage.scan_for(&self.detector, depth, region, &start));
        let crowded = from.is_some_and(|pose| stage.is_crowded(depth, &pose));
        let (measured, tracked) = match &scan {
            Some((start, _)) if crowded => {
                self.tracker.reset();
                (Some(*start), None)
            }
            Some((start, cloud)) => {
                let update = self.tracker.update_from(cloud, Some(*start)).ok();
                let pose = update
                    .as_ref()
                    .filter(|update| update.trusted)
                    .map(|update| update.pose);
                (pose, update)
            }
            None => (None, None),
        };
        let pose = self.hold.judge(
            &self.detector,
            &stage.camera,
            (depth, region),
            measured,
            !crowded,
        );
        (scan.map(|(_, cloud)| cloud), Answer { pose, tracked })
    }
}

/// A frame's outcome: the pose answered, and what the tracker said of it when it
/// had a say.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    pub pose: Option<RigidTransform>,
    pub tracked: Option<Update>,
}
