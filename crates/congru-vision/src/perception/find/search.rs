use congru::RigidTransform;

use super::{Finder, Found};
use crate::{Camera, DepthImage, Information, Observed, render};

/// The votes scored best at the finder's step, refined along rays and scored
/// again, closer to the noise.
const REFINED: usize = 128;
/// The final score reads at this share of the step.
const TIGHT: f64 = 0.5;
/// Iterations along rays before a vote is judged at all: a raw vote, known to
/// a bin, scores too low to survive.
const FIRST_ITERATIONS: u32 = 4;

impl Finder {
    /// The pose the frame supports best. Votes only propose: their count
    /// favours large surfaces. Each is brought a little onto the frame and judged
    /// loosely; only the survivors are refined and judged closely.
    pub(super) fn search(&self, depth: &DepthImage, camera: &Camera) -> Option<Found> {
        let (frame, index) = self.frame_of(depth, camera)?;
        let step = self.table.step;
        let loose = Observed::new(depth.clone(), camera, step);
        let tight = Observed::new(depth.clone(), camera, step * TIGHT);
        let mut voted: Vec<(f64, RigidTransform)> = frame
            .iter()
            .step_by(super::REFERENCE_STRIDE)
            .flat_map(|reference| self.votes(reference, &frame, &index))
            .map(|pose| self.refined_by(depth, camera, &pose, FIRST_ITERATIONS))
            .map(|pose| (self.score(&loose, camera, &pose, step), pose))
            .collect();
        voted.sort_by(|left, right| right.0.total_cmp(&left.0));
        let (_, pose) = voted
            .iter()
            .take(REFINED)
            .map(|(_, pose)| self.refined(depth, camera, pose))
            .map(|pose| (self.score(&tight, camera, &pose, step * TIGHT), pose))
            .max_by(|left, right| left.0.total_cmp(&right.0))?;
        Some(Found {
            pose,
            agreement: self.confirmed(&tight, camera, &pose),
        })
    }

    /// Every raw vote at `stride`, for measuring what the search can offer.
    #[must_use]
    pub fn raw_votes(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        stride: usize,
    ) -> Vec<RigidTransform> {
        let Some((frame, index)) = self.frame_of(depth, camera) else {
            return Vec::new();
        };
        frame
            .iter()
            .step_by(stride)
            .flat_map(|reference| self.votes(reference, &frame, &index))
            .collect()
    }

    /// The share of what the model could say at `pose` that the frame says.
    /// Near one when the whole of it is there, near zero when it is not.
    #[must_use]
    pub fn confirmed(&self, seen: &Observed, camera: &Camera, pose: &RigidTransform) -> f64 {
        let tolerance = self.table.step * TIGHT;
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        let itself = Observed::new(rendered.clone(), camera, tolerance);
        let whole = Information::of(&itself, &rendered, camera, tolerance).mean();
        if whole > 0.0 {
            self.score(seen, camera, pose, tolerance) / whole
        } else {
            0.0
        }
    }
}
