mod faces;
mod pairs;
mod rays;
mod search;
mod turned;
mod votes;

use congru::{Distance, Error, RigidTransform, TriangleMesh};

use crate::{Camera, DepthImage, Information, Observed, ViewScore, render};
use pairs::{Oriented, PairTable};
use rays::along_rays;

/// A frame point out of this many is a reference that votes.
const REFERENCE_STRIDE: usize = 4;

/// Pairs farther apart along a ray than the model's size over this are
/// left out while refining.
const RAY_GATE_SHARE: f64 = 5.0;
/// Iterations that bring a voted candidate, known to a bin, onto the frame.
const REFINE_ITERATIONS: u32 = 10;

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
    points_centre: congru::Point,
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
        let points_centre = congru::Point::from(
            points
                .iter()
                .fold(congru::Vector::zeros(), |sum, oriented| {
                    sum + oriented.point.coords
                })
                / crate::pixel::coordinate(points.len().max(1)),
        );
        Ok(Self {
            points_centre,
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

    /// `pose`, already close, averaged onto the returns at the model's depth: only
    /// those within `gate` count, so what stands around has no say, and many
    /// pixels give the precision a verdict per pixel cannot.
    #[must_use]
    pub fn settled(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
        gate: f64,
    ) -> RigidTransform {
        along_rays(depth, camera, &self.points, pose, gate, REFINE_ITERATIONS)
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
}
