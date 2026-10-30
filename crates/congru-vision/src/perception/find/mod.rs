mod check;
mod pairs;

use congru::{Distance, Error, NearestNeighbors, PointCloud, RigidTransform, Target, TriangleMesh};

use crate::{Camera, DepthImage, back_project};
use check::agreement;
use pairs::{ANGLE_BINS, Oriented, PairTable, key, local_frame, turn, turn_bin, turn_of};

/// A frame point out of this many is a reference that votes.
const REFERENCE_STRIDE: usize = 4;
/// Neighbours a frame normal is fitted to.
const NORMAL_NEIGHBORS: usize = 10;
/// The best voted poses checked against the depth.
const CANDIDATES: usize = 48;

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
    /// less the share it sees behind the model.
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
        let mut voted: Vec<(usize, RigidTransform)> = frame
            .iter()
            .step_by(REFERENCE_STRIDE)
            .map(|reference| self.vote(reference, &frame, &index))
            .collect();
        voted.sort_by_key(|(count, _)| std::cmp::Reverse(*count));
        voted
            .iter()
            .take(CANDIDATES)
            .map(|(_, pose)| Found {
                pose: *pose,
                agreement: agreement(depth, camera, &self.model, pose, self.table.step),
            })
            .max_by(|left, right| left.agreement.total_cmp(&right.agreement))
            .filter(|found| found.agreement >= least)
    }

    /// The votes of one frame point and its neighbours: the peak's count,
    /// and the pose it stands for.
    fn vote(
        &self,
        reference: &Oriented,
        frame: &[Oriented],
        index: &NearestNeighbors,
    ) -> (usize, RigidTransform) {
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
        let (peak, count) = ballot
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map_or((0, 0), |(peak, count)| (peak, *count));
        let spin = RigidTransform::from_axis_angle(
            congru::Vector::x() * turn_of(peak % ANGLE_BINS),
            [0.0; 3],
        );
        let pose = local_frame(&self.points[peak / ANGLE_BINS])
            .then(&spin)
            .then(&local.inverse());
        (count, pose)
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
