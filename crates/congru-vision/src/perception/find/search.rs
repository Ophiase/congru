use congru::{NearestNeighbors, RigidTransform};

use super::faces::{frame_patches, model_faces, placements};
use super::pairs::Oriented;
use super::{Finder, Found};
use crate::perception::noise_under;
use crate::{Camera, DepthImage, Information, Observed, ViewScore, render};

/// The votes scored best at the finder's step, refined along rays and scored
/// again, closer to the noise.
const REFINED: usize = 128;
/// The final score reads at this share of the step.
pub(super) const TIGHT: f64 = 0.5;
/// A tolerance spans this many deviations of the noise: enough to keep most hits,
/// little enough to tell a right pose from one a few degrees off, as measured on
/// the score's ranking.
const NOISE_SPAN: f64 = 1.25;
/// Iterations along rays before a vote is judged at all: a raw vote, known to
/// a bin, scores too low to survive.
pub(super) const FIRST_ITERATIONS: u32 = 4;
/// A search stays on this many threads, to leave the machine usable.
const MOST_THREADS: usize = 4;

impl Finder {
    /// The pose the frame supports best. Votes only propose: their count
    /// favours large surfaces. Each is brought a little onto the frame and judged
    /// loosely; only the survivors are refined and judged closely.
    ///
    /// `region` is what proposes, a frame cut to a region of interest or the whole
    /// of it; poses are judged on the whole `depth`, since beyond the box's border
    /// nothing was measured, and that is not an edge of the object.
    pub(super) fn search(
        &self,
        depth: &DepthImage,
        region: &DepthImage,
        camera: &Camera,
    ) -> Option<Found> {
        let (frame, index) = self.frame_of(region, camera)?;
        let tight = Observed::new(depth.clone(), camera, self.close_to(depth, region));
        let wide = tight.tolerance() / TIGHT;
        let loose = Observed::new(depth.clone(), camera, wide);
        let references: Vec<&Oriented> = frame.iter().step_by(super::REFERENCE_STRIDE).collect();
        let brought = shared(&references, |reference| {
            self.votes(reference, &frame, &index)
                .iter()
                .map(|pose| self.refined_by(depth, camera, pose, FIRST_ITERATIONS))
                .collect()
        });
        let turned = shared(&self.orientations_in(&frame), |pose| {
            vec![self.refined_by(depth, camera, pose, FIRST_ITERATIONS)]
        });
        let candidates = [brought, turned, self.face_placements(&frame, &index)].concat();
        let mut voted: Vec<(f64, RigidTransform)> = shared(&candidates, |pose| {
            vec![(self.score(&loose, camera, pose, wide), *pose)]
        });
        voted.sort_by(|left, right| right.0.total_cmp(&left.0));
        let best: Vec<(f64, RigidTransform)> = voted.iter().take(REFINED).copied().collect();
        let judged = shared(&best, |(_, pose)| {
            [*pose, self.refined(depth, camera, pose)]
                .iter()
                .map(|pose| (self.score(&tight, camera, pose, tight.tolerance()), *pose))
                .collect()
        });
        let (_, pose) = judged
            .into_iter()
            .max_by(|left, right| left.0.total_cmp(&right.0))?;
        Some(Found {
            pose,
            agreement: self.confirmed(&tight, camera, &pose),
        })
    }

    /// Every pose laying a flat face of the model on a flat region of the frame:
    /// what votes cannot offer, since a flat face gives the same pairs everywhere.
    pub(super) fn face_placements(
        &self,
        frame: &[Oriented],
        index: &NearestNeighbors,
    ) -> Vec<RigidTransform> {
        let step = self.table.step;
        let faces = model_faces(&self.points, step);
        frame_patches(frame, index, step)
            .iter()
            .flat_map(|patch| faces.iter().flat_map(|face| placements(patch, face, step)))
            .collect()
    }

    /// The face placements of a frame, for measuring.
    #[must_use]
    pub fn placed_faces(&self, depth: &DepthImage, camera: &Camera) -> Vec<RigidTransform> {
        self.frame_of(depth, camera)
            .map_or_else(Vec::new, |(frame, index)| {
                self.face_placements(&frame, &index)
            })
    }

    /// The share of the model's pixels at `pose` whose ray went through it, read
    /// at the tolerance the noise there asks for.
    #[must_use]
    pub fn through_share(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        let view = ViewScore::of(depth, &rendered, camera, self.close_to(depth, &rendered));
        let total = view.support + view.violation + view.occluded;
        if total > 0.0 {
            view.violation / total
        } else {
            0.0
        }
    }

    /// The tolerance that reads `depth` close to its noise: half the step at
    /// least, and more where the pixels under `mask` differ by more.
    pub(super) fn close_to(&self, depth: &DepthImage, mask: &DepthImage) -> f64 {
        let floor = self.table.step * TIGHT;
        noise_under(depth, mask).map_or(floor, |noise| floor.max(NOISE_SPAN * noise))
    }

    /// `depth` read at the tolerance its own noise asks for around `pose`.
    #[must_use]
    pub fn observed_at(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
    ) -> Observed {
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        Observed::new(depth.clone(), camera, self.close_to(depth, &rendered))
    }

    /// As [`Finder::confirmed`], reading `depth` first.
    #[must_use]
    pub fn confirmed_at(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        self.confirmed(&self.observed_at(depth, camera, pose), camera, pose)
    }

    /// The share of what the model could say at `pose` that the frame says.
    /// Near one when the whole of it is there, near zero when it is not.
    #[must_use]
    pub fn confirmed(&self, seen: &Observed, camera: &Camera, pose: &RigidTransform) -> f64 {
        let tolerance = seen.tolerance();
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        let alone = rendered.map(|value| {
            if value.is_finite() {
                value
            } else {
                f64::INFINITY
            }
        });
        let itself = Observed::new(alone, camera, tolerance);
        let whole = Information::of(&itself, &rendered, camera, tolerance).mean();
        if whole > 0.0 {
            self.score(seen, camera, pose, tolerance) / whole
        } else {
            0.0
        }
    }
}

/// `work` over `items` on at most [`MOST_THREADS`] threads, results in order.
pub(super) fn shared<Item: Sync, Answer: Send>(
    items: &[Item],
    work: impl Fn(&Item) -> Vec<Answer> + Sync,
) -> Vec<Answer> {
    let threads =
        std::thread::available_parallelism().map_or(1, |count| count.get().min(MOST_THREADS));
    let share = items.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let running: Vec<_> = items
            .chunks(share)
            .map(|chunk| scope.spawn(|| chunk.iter().flat_map(&work).collect::<Vec<Answer>>()))
            .collect();
        running
            .into_iter()
            .flat_map(|thread| thread.join().unwrap_or_default())
            .collect()
    })
}
