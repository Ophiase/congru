use std::collections::VecDeque;

use congru::RigidTransform;
use congru_vision::{Camera, Finder};

use super::Sight;
use crate::{MotionFilter, MotionModel};

/// How many frames set the level a frame's confirmation is read against.
const RECENT: usize = 9;
/// A frame is believed down to this share of the recent level.
const LEAST_OF_RECENT: f64 = 0.5;
/// Above this share of rays through the forecast, it is refuted, not hidden.
const MOST_THROUGH: f64 = 0.15;
/// After this many frames on the forecast alone, the object is lost.
const MOST_COASTED: usize = 20;
/// After this many frames on the forecast, it is searched again whatever the rays
/// say: a wrong pose lying flat on a table is neither hidden nor refuted.
const LOOK_AGAIN: usize = 3;

/// What goes on when a frame is not believed: the trajectory holds through what
/// hides the object, and is searched again when the frame refutes it.
#[derive(Debug)]
pub struct Hold {
    filter: MotionFilter,
    recent: VecDeque<f64>,
    coasted: usize,
}

impl Hold {
    #[must_use]
    pub fn new(motion: MotionModel) -> Self {
        Self {
            filter: MotionFilter::of(motion),
            recent: VecDeque::new(),
            coasted: 0,
        }
    }

    /// Where the object should be at this frame.
    #[must_use]
    pub fn forecast(&self) -> Option<RigidTransform> {
        self.filter.predict()
    }

    fn believed(&self, confirmed: f64) -> bool {
        let mut levels: Vec<f64> = self.recent.iter().copied().collect();
        levels.sort_by(f64::total_cmp);
        let level = levels.get(levels.len() / 2).copied().unwrap_or(0.0);
        confirmed >= 0.0 && confirmed >= LEAST_OF_RECENT * level
    }

    fn accept(&mut self, pose: &RigidTransform, confirmed: f64) -> RigidTransform {
        let corrected = self.filter.correct(pose, &[]);
        self.recent.push_back(confirmed);
        if self.recent.len() > RECENT {
            self.recent.pop_front();
        }
        self.coasted = 0;
        corrected
    }

    /// The pose to answer for this frame, from what was `measured` on it; `None`
    /// says the object is lost. A pose along rays is noisy and answers smoothed; one
    /// `aligned` on points answers as it is, since a jolt moves it at once.
    pub fn judge(
        &mut self,
        finder: &Finder,
        camera: &Camera,
        sight: Sight<'_>,
        measured: Option<RigidTransform>,
        aligned: bool,
    ) -> Option<RigidTransform> {
        if let Some(pose) = measured {
            let confirmed = finder.confirmed_at(sight.depth, camera, &pose);
            if self.believed(confirmed) {
                let corrected = self.accept(&pose, confirmed);
                return Some(if aligned { pose } else { corrected });
            }
        }
        let forecast = self.filter.predict()?;
        if self.coasted >= LOOK_AGAIN
            || finder.through_share(sight.depth, camera, &forecast) > MOST_THROUGH
        {
            let turned = finder
                .turned_again(sight.depth, camera, &forecast)
                .map(|found| found.pose);
            let again = self.restart(finder, camera, sight, turned).or_else(|| {
                let found = finder.located_in(sight.depth, sight.region, camera);
                self.restart(finder, camera, sight, found)
            });
            if again.is_some() {
                return again;
            }
        }
        self.coasted += 1;
        if self.coasted > MOST_COASTED {
            self.filter.reset();
            self.recent.clear();
            self.coasted = 0;
            return None;
        }
        self.filter.correct(&forecast, &[]);
        Some(forecast)
    }

    /// Starts the trajectory over from `found`, when the frame believes it.
    fn restart(
        &mut self,
        finder: &Finder,
        camera: &Camera,
        sight: Sight<'_>,
        found: Option<RigidTransform>,
    ) -> Option<RigidTransform> {
        let found = found?;
        let confirmed = finder.confirmed_at(sight.depth, camera, &found);
        self.believed(confirmed).then(|| {
            self.filter.reset();
            self.accept(&found, confirmed)
        })
    }
}
