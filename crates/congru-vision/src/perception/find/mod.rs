mod check;
mod pairs;
mod rays;

use congru::{Distance, Error, NearestNeighbors, PointCloud, RigidTransform, Target, TriangleMesh};

use crate::{Camera, DepthImage, back_project};
use check::{agreement, share_in_frame};
use pairs::{ANGLE_BINS, Oriented, PairTable, key, local_frame, turn, turn_bin, turn_of};
use rays::along_rays;

/// A frame point out of this many is a reference that votes.
const REFERENCE_STRIDE: usize = 4;
/// Neighbours a frame normal is fitted to.
const NORMAL_NEIGHBORS: usize = 10;

/// Pairs farther apart along a ray than the model's size over this are
/// left out while refining.
const RAY_GATE_SHARE: f64 = 10.0;
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
}

/// Where the object was found, and how well the frame agrees with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Found {
    /// Takes the model into the camera's frame.
    pub pose: RigidTransform,
    /// The share of the model's pixels the frame sees at the model's depth,
    /// less the share it sees behind the model, times the share of the model
    /// inside the image.
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

    /// The best pose of the model in `depth`, when one agrees with at least
    /// `least` of the frame; `None` says the object was not found.
    #[must_use]
    pub fn find(&self, depth: &DepthImage, camera: &Camera, least: f64) -> Option<Found> {
        let step = Distance::new(self.table.step).ok()?;
        let thinned = PointCloud::new(back_project(depth, camera))
            .ok()?
            .voxel_downsample(step);
        let scene = Target::with_estimated_normals(thinned, NORMAL_NEIGHBORS).ok()?;
        let frame: Vec<Oriented> = facing_camera(&scene).ok()?;
        let index = NearestNeighbors::new(scene.cloud()).ok()?;
        frame
            .iter()
            .step_by(REFERENCE_STRIDE)
            .map(|reference| self.vote(reference, &frame, &index))
            .map(|pose| self.refined(depth, camera, &pose))
            .map(|pose| Found {
                pose,
                agreement: agreement(depth, camera, &self.model, &pose, self.table.step)
                    * share_in_frame(
                        camera,
                        &pose,
                        self.points.iter().map(|oriented| &oriented.point),
                    ),
            })
            .max_by(|left, right| left.agreement.total_cmp(&right.agreement))
            .filter(|found| found.agreement >= least)
    }

    /// A voted pose, known only to a bin, brought onto the frame along its
    /// rays.
    fn refined(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        pose: &RigidTransform,
    ) -> RigidTransform {
        along_rays(
            depth,
            camera,
            &self.points,
            pose,
            self.diameter / RAY_GATE_SHARE,
            REFINE_ITERATIONS,
        )
    }

    /// The pose the votes of one frame point and its neighbours stand for.
    fn vote(
        &self,
        reference: &Oriented,
        frame: &[Oriented],
        index: &NearestNeighbors,
    ) -> RigidTransform {
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
        let peak = ballot
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map_or(0, |(peak, _)| peak);
        let spin = RigidTransform::from_axis_angle(
            congru::Vector::x() * turn_of(peak % ANGLE_BINS),
            [0.0; 3],
        );
        local_frame(&self.points[peak / ANGLE_BINS])
            .then(&spin)
            .then(&local.inverse())
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
