use congru::{NearestNeighbors, RigidTransform, Vector};

use super::faces::{frame_patches, model_faces, placements};
use super::pairs::Oriented;
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
/// A search stays on this many threads, to leave the machine usable.
const MOST_THREADS: usize = 4;
/// The first moves of a polish, in radians and in the unit of the depths, and
/// how they shrink when none helps.
const POLISH_TURN: f64 = 0.02;
const POLISH_SHIFT: f64 = 0.004;
const POLISH_SHRINK: f64 = 0.5;
const POLISH_ROUNDS: u32 = 6;

impl Finder {
    /// The pose the frame supports best. Votes only propose: their count
    /// favours large surfaces. Each is brought a little onto the frame and judged
    /// loosely; only the survivors are refined and judged closely.
    pub(super) fn search(&self, depth: &DepthImage, camera: &Camera) -> Option<Found> {
        let (frame, index) = self.frame_of(depth, camera)?;
        let step = self.table.step;
        let loose = Observed::new(depth.clone(), camera, step);
        let tight = Observed::new(depth.clone(), camera, step * TIGHT);
        let references: Vec<&Oriented> = frame.iter().step_by(super::REFERENCE_STRIDE).collect();
        let brought = shared(&references, |reference| {
            self.votes(reference, &frame, &index)
                .iter()
                .map(|pose| self.refined_by(depth, camera, pose, FIRST_ITERATIONS))
                .collect()
        });
        let candidates = [brought, self.face_placements(&frame, &index)].concat();
        let mut voted: Vec<(f64, RigidTransform)> = shared(&candidates, |pose| {
            vec![(self.score(&loose, camera, pose, step), *pose)]
        });
        voted.sort_by(|left, right| right.0.total_cmp(&left.0));
        let best: Vec<(f64, RigidTransform)> = voted.iter().take(REFINED).copied().collect();
        let judged = shared(&best, |(_, pose)| {
            [*pose, self.refined(depth, camera, pose)]
                .iter()
                .map(|pose| (self.score(&tight, camera, pose, step * TIGHT), *pose))
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

    /// `start` improved by small moves the rays support, the moves shrinking
    /// when none helps: rays through the model pin what hits leave free.
    #[must_use]
    pub fn polished(
        &self,
        seen: &Observed,
        camera: &Camera,
        start: &RigidTransform,
    ) -> RigidTransform {
        let tolerance = self.table.step * TIGHT;
        let (mut pose, mut best) = (*start, self.score(seen, camera, start, tolerance));
        let (mut turn, mut shift) = (POLISH_TURN, POLISH_SHIFT);
        for _ in 0..POLISH_ROUNDS {
            let mut better = false;
            for axis in [Vector::x(), Vector::y(), Vector::z()] {
                for sign in [1.0, -1.0] {
                    let moved = axis * sign;
                    let centre = pose.apply(&self.points_centre);
                    let spin = RigidTransform::from_axis_angle(Vector::zeros(), -centre.coords)
                        .then(&RigidTransform::from_axis_angle(moved * turn, [0.0; 3]))
                        .then(&RigidTransform::from_axis_angle(
                            Vector::zeros(),
                            centre.coords,
                        ));
                    let slide = moved * shift;
                    let slid = RigidTransform::from_axis_angle(
                        Vector::zeros(),
                        [slide.x, slide.y, slide.z],
                    );
                    for trial in [pose.then(&spin), pose.then(&slid)] {
                        let value = self.score(seen, camera, &trial, tolerance);
                        if value > best {
                            (pose, best, better) = (trial, value, true);
                        }
                    }
                }
            }
            if !better {
                turn *= POLISH_SHRINK;
                shift *= POLISH_SHRINK;
            }
        }
        pose
    }

    /// As [`Finder::confirmed`], reading `depth` first.
    #[must_use]
    pub fn confirmed_at(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        let seen = Observed::new(depth.clone(), camera, self.table.step * TIGHT);
        self.confirmed(&seen, camera, pose)
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

/// `work` over `items` on at most [`MOST_THREADS`] threads, results in order.
fn shared<Item: Sync, Answer: Send>(
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
