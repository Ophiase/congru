use congru::{Distance, PointCloud, RigidTransform};
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
    latest: Option<Update>,
}

impl Follower {
    pub(super) fn new(stage: &Stage) -> anyhow::Result<Self> {
        let view = CameraView::new(&stage.part, stage.camera, Distance::new(MODEL_SPACING)?)?;
        let settings = Settings::for_part(PART_SPAN)?;
        Ok(Self {
            tracker: Tracker::new("tracker", view, settings),
            detector: Detector::new(&stage.part, PART_SPAN)?,
            hold: Hold::new(settings.gains),
            latest: None,
        })
    }

    pub(super) fn forecast(&self) -> Option<RigidTransform> {
        self.hold.forecast()
    }

    /// One frame from the pose it starts `from`: the scan given to the tracker,
    /// and the answer.
    pub(super) fn step(
        &mut self,
        stage: &Stage,
        (depth, region): (&DepthImage, &DepthImage),
        from: Option<RigidTransform>,
    ) -> (Option<PointCloud>, Option<Update>) {
        let scan = from.and_then(|start| stage.scan_for(&self.detector, depth, region, &start));
        let crowded = from.is_some_and(|pose| stage.is_crowded(depth, &pose));
        let fresh = scan.as_ref().and_then(|(start, cloud)| {
            let update = self.tracker.update_from(cloud, Some(*start)).ok()?;
            Some(if crowded {
                Update {
                    pose: *start,
                    trusted: true,
                    ..update
                }
            } else {
                update
            })
        });
        let measured = fresh
            .as_ref()
            .filter(|update| update.trusted)
            .map(|update| update.pose);
        let answer = self.hold.judge(
            &self.detector,
            &stage.camera,
            (depth, region),
            measured,
            !crowded,
        );
        self.latest = fresh.or_else(|| self.latest.take());
        let update = answer.and_then(|pose| {
            self.latest.as_ref().map(|update| Update {
                pose,
                ..update.clone()
            })
        });
        (scan.map(|(_, cloud)| cloud), update)
    }
}
