//! An object followed through depth frames in a whole scene: the region says
//! roughly where, the frame judges, and a frame that says nothing is held through.

mod hold;
mod scan;

use congru::{Aligner, Error, PointCloud, RigidTransform, TriangleMesh};
use congru_vision::{Camera, DepthImage, Finder};

use crate::view::CameraView;
use crate::{Settings, Tracker, Update};
pub use hold::Hold;
use scan::Scope;
pub use scan::crowding;

/// The model's sampling, as a share of its size.
const SAMPLE_SHARE: f64 = 1.0 / 80.0;

/// One depth frame, and the same frame cut to a region of interest: the rectangle a
/// 2D detector gives, or the frame itself without one.
#[derive(Debug, Clone, Copy)]
pub struct Sight<'frame> {
    pub depth: &'frame DepthImage,
    pub region: &'frame DepthImage,
}

/// A frame's outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// `None` says the object is lost, or not found yet.
    pub pose: Option<RigidTransform>,
    /// What the tracker said, on a frame with little else than the object.
    pub tracked: Option<Update>,
    /// The points aligned on.
    pub scan: Option<PointCloud>,
}

/// Everything one run remembers from a frame to the next.
#[derive(Debug)]
pub struct Follower {
    tracker: Tracker<CameraView>,
    finder: Finder,
    hold: Hold,
    scope: Scope,
    camera: Camera,
    search_every: usize,
    frame: usize,
}

impl Follower {
    /// `span` is the object's size, in the unit of the depths.
    ///
    /// # Errors
    /// When the model cannot be sampled at a share of `span`.
    pub fn new(
        part: &TriangleMesh,
        camera: Camera,
        span: f64,
        settings: Settings,
    ) -> Result<Self, Error> {
        let spacing = congru::Distance::new(span * SAMPLE_SHARE)?;
        Ok(Self {
            tracker: Tracker::new("tracker", CameraView::new(part, camera, spacing)?, settings),
            finder: Finder::for_span(part, span)?,
            hold: Hold::new(settings.motion),
            scope: Scope::new(part, span),
            camera,
            search_every: 1,
            frame: 0,
        })
    }

    /// A lost object is searched at one frame in `frames`: a search costs seconds.
    #[must_use]
    pub fn searching_every(self, frames: usize) -> Self {
        Self {
            search_every: frames.max(1),
            ..self
        }
    }

    #[must_use]
    pub const fn finder(&self) -> &Finder {
        &self.finder
    }

    /// Where the object should be at this frame.
    #[must_use]
    pub fn forecast(&self) -> Option<RigidTransform> {
        self.hold.forecast()
    }

    /// One frame, from the forecast, or from a search when there is none.
    pub fn step(&mut self, sight: Sight<'_>) -> Answer {
        let from = self.forecast().or_else(|| {
            self.frame
                .is_multiple_of(self.search_every)
                .then(|| {
                    self.finder
                        .located_in(sight.depth, sight.region, &self.camera)
                })
                .flatten()
        });
        self.step_from(sight, from)
    }

    /// One frame, from `from`. In a crowded frame the rays have the say, and the
    /// tracker, which only clutter would feed, starts over when the frame clears.
    pub fn step_from(&mut self, sight: Sight<'_>, from: Option<RigidTransform>) -> Answer {
        self.frame += 1;
        let scan =
            from.and_then(|start| self.scope.scan(&self.finder, &self.camera, sight, &start));
        let (measured, tracked, aligned) = match &scan {
            Some(scan) if scan.crowded => {
                self.tracker.reset();
                (
                    Some(self.near(sight.depth, &scan.points, &scan.start)),
                    None,
                    false,
                )
            }
            Some(scan) => {
                let update = self
                    .tracker
                    .update_from(&scan.points, Some(scan.start))
                    .ok();
                let pose = update
                    .as_ref()
                    .filter(|update| update.trusted)
                    .map(Update::measured);
                (pose, update, true)
            }
            None => (None, None, false),
        };
        let pose = self
            .hold
            .judge(&self.finder, &self.camera, sight, measured, aligned);
        Answer {
            pose,
            tracked,
            scan: scan.map(|scan| scan.points),
        }
    }

    /// `start` aligned on the points close to it, when the frame prefers that: rays
    /// give the place and points the precision, and no clutter reaches so short a
    /// gate from a start this close.
    fn near(
        &self,
        depth: &DepthImage,
        points: &PointCloud,
        start: &RigidTransform,
    ) -> RigidTransform {
        self.tracker
            .aligned_near(points, start)
            .ok()
            .filter(|pose| {
                self.finder.confirmed_at(depth, &self.camera, pose)
                    >= self.finder.confirmed_at(depth, &self.camera, start)
            })
            .unwrap_or(*start)
    }
}
