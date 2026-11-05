mod check;
mod pairs;
mod rays;
mod search;

use congru::{Distance, Error, NearestNeighbors, PointCloud, RigidTransform, Target, TriangleMesh};

use crate::{Camera, DepthImage, Information, Observed, ViewScore, back_project, render};
use check::{agreement, share_in_frame};
use pairs::{ANGLE_BINS, Oriented, PairTable, key, local_frame, turn, turn_bin, turn_of};
use rays::along_rays;

/// A frame point out of this many is a reference that votes.
const REFERENCE_STRIDE: usize = 4;
/// Neighbours a frame normal is fitted to.
const NORMAL_NEIGHBORS: usize = 10;

/// Pairs farther apart along a ray than the model's size over this are
/// left out while refining.
const RAY_GATE_SHARE: f64 = 5.0;
/// Iterations that bring a voted candidate, known to a bin, onto the frame.
const REFINE_ITERATIONS: u32 = 10;
/// Peaks a reference proposes: clutter among its neighbours splits its votes,
/// so the right pose is not always its highest.
const PEAKS: usize = 4;

/// A known object searched in a whole depth frame by its shape alone, with
/// [point pair features] voted, each candidate checked against the depth.
///
/// [point pair features]: https://doi.org/10.1109/CVPR.2010.5540108
#[derive(Debug)]
pub struct Finder {
    model: TriangleMesh,
    points: Vec<Oriented>,
    table: PairTable,
    diameter: f64,
}

/// Where the object was found, and how well the frame agrees with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Found {
    /// Takes the model into the camera's frame.
    pub pose: RigidTransform,
    /// The share of what the model could say at this pose that the frame
    /// says, less what it refutes.
    pub agreement: f64,
}

impl Finder {
    /// `step` sets both the sampling and the distance bins: about a
    /// twentieth of the model's size.
    ///
    /// # Errors
    /// When the model cannot be sampled at `step`.
    pub fn new(model: &TriangleMesh, step: Distance) -> Result<Self, Error> {
        let surface = model.sample_surface(step)?;
        let points: Vec<Oriented> = surface
            .cloud()
            .points()
            .iter()
            .zip(surface.normals())
            .map(|(point, normal)| Oriented {
                point: *point,
                normal: normal.into_inner(),
            })
            .collect();
        let diameter = points
            .iter()
            .flat_map(|first| {
                points
                    .iter()
                    .map(|second| (second.point - first.point).norm())
            })
            .fold(0.0, f64::max);
        Ok(Self {
            model: model.clone(),
            table: PairTable::new(&points, step.get()),
            points,
            diameter,
        })
    }

    /// The best pose of the model in `depth`, when the frame confirms at
    /// least `least` of what the model could say there; `None` says the object
    /// was not found.
    #[must_use]
    pub fn find(&self, depth: &DepthImage, camera: &Camera, least: f64) -> Option<Found> {
        self.search(depth, camera)
            .filter(|found| found.agreement >= least)
    }

    /// Every vote, refined along its rays, for measuring what the search
    /// can offer.
    #[must_use]
    pub fn proposals(&self, depth: &DepthImage, camera: &Camera) -> Vec<RigidTransform> {
        let Some((frame, index)) = self.frame_of(depth, camera) else {
            return Vec::new();
        };
        frame
            .iter()
            .step_by(REFERENCE_STRIDE)
            .filter_map(|reference| self.votes(reference, &frame, &index).into_iter().next())
            .map(|pose| self.refined(depth, camera, &pose))
            .collect()
    }

    /// The frame's oriented points, thinned to the step, with their index.
    fn frame_of(
        &self,
        depth: &DepthImage,
        camera: &Camera,
    ) -> Option<(Vec<Oriented>, NearestNeighbors)> {
        let step = Distance::new(self.table.step).ok()?;
        let thinned = PointCloud::new(back_project(depth, camera))
            .ok()?
            .voxel_downsample(step);
        let scene = Target::with_estimated_normals(thinned, NORMAL_NEIGHBORS).ok()?;
        let frame = facing_camera(&scene).ok()?;
        let index = NearestNeighbors::new(scene.cloud()).ok()?;
        Some((frame, index))
    }

    /// What the frame says of `pose`: the information of what it confirms,
    /// less the area it refutes. An absolute amount, for comparing poses.
    #[must_use]
    pub fn score(
        &self,
        seen: &Observed,
        camera: &Camera,
        pose: &RigidTransform,
        tolerance: f64,
    ) -> f64 {
        let rendered = render(camera, &[(&self.model, *pose)]).depth;
        Information::of(seen, &rendered, camera, tolerance).mean()
            - ViewScore::of(&seen.depth, &rendered, camera, tolerance).violation
    }

    /// The score of today's detector, for comparison.
    #[must_use]
    pub fn agreement(&self, depth: &DepthImage, camera: &Camera, pose: &RigidTransform) -> f64 {
        agreement(depth, camera, &self.model, pose, self.table.step)
            * share_in_frame(
                camera,
                pose,
                self.points.iter().map(|oriented| &oriented.point),
            )
    }

    /// A voted pose, known only to a bin, brought onto the frame along its
    /// rays.
    #[must_use]
    pub fn refined(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
    ) -> RigidTransform {
        self.refined_by(depth, camera, pose, REFINE_ITERATIONS)
    }

    /// As [`Finder::refined`], for this many iterations.
    #[must_use]
    pub fn refined_by(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
        iterations: u32,
    ) -> RigidTransform {
        let widest = self.diameter / RAY_GATE_SHARE;
        rays::GATE_SHRINKS.iter().fold(*pose, |pose, shrink| {
            along_rays(
                depth,
                camera,
                &self.points,
                &pose,
                widest / shrink,
                iterations.div_ceil(2),
            )
        })
    }

    /// The poses the votes of one frame point and its neighbours stand for:
    /// the highest peaks, one per model point, highest first.
    fn votes(
        &self,
        reference: &Oriented,
        frame: &[Oriented],
        index: &NearestNeighbors,
    ) -> Vec<RigidTransform> {
        let local = local_frame(reference);
        let mut ballot = vec![0_usize; self.points.len() * ANGLE_BINS];
        for neighbor in index.within(&reference.point, self.diameter) {
            let other = &frame[neighbor.index];
            let Some(hits) = self
                .table
                .entries
                .get(&key(reference, other, self.table.step))
            else {
                continue;
            };
            let angle = turn(&local, &other.point);
            for (model, model_angle) in hits {
                ballot[model * ANGLE_BINS + turn_bin(angle - model_angle)] += 1;
            }
        }
        let mut peaks: Vec<(usize, usize)> = ballot
            .chunks(ANGLE_BINS)
            .enumerate()
            .filter_map(|(model, turns)| {
                let (bin, count) = turns.iter().enumerate().max_by_key(|(_, count)| **count)?;
                (*count > 0).then_some((*count, model * ANGLE_BINS + bin))
            })
            .collect();
        peaks.sort_unstable_by_key(|(count, _)| std::cmp::Reverse(*count));
        peaks
            .into_iter()
            .take(PEAKS)
            .map(|(_, peak)| {
                let spin = RigidTransform::from_axis_angle(
                    congru::Vector::x() * turn_of(peak % ANGLE_BINS),
                    [0.0; 3],
                );
                local_frame(&self.points[peak / ANGLE_BINS])
                    .then(&spin)
                    .then(&local.inverse())
            })
            .collect()
    }
}

/// The frame's points with their normals, each turned toward the camera.
fn facing_camera(target: &Target) -> Result<Vec<Oriented>, Error> {
    let normals = target.normals().ok_or(Error::MissingNormals)?;
    Ok(target
        .cloud()
        .points()
        .iter()
        .zip(normals)
        .map(|(point, normal)| {
            let normal = normal.into_inner();
            Oriented {
                point: *point,
                normal: if normal.dot(&point.coords) > 0.0 {
                    -normal
                } else {
                    normal
                },
            }
        })
        .collect())
}
