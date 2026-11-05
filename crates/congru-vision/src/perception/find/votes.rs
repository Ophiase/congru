use congru::{Distance, Error, NearestNeighbors, PointCloud, RigidTransform, Target};

use super::Finder;
use super::pairs::{ANGLE_BINS, Oriented, key, local_frame, turn, turn_bin, turn_of};
use crate::{Camera, DepthImage, back_project};

/// Neighbours a frame normal is fitted to.
const NORMAL_NEIGHBORS: usize = 10;
/// Peaks a reference proposes: clutter among its neighbours splits its votes,
/// so the right pose is not always its highest.
const PEAKS: usize = 4;

impl Finder {
    /// The frame's oriented points, thinned to the step, with their index.
    pub(super) fn frame_of(
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

    /// The poses the votes of one frame point and its neighbours stand for:
    /// the highest peaks, one per model point, highest first.
    pub(super) fn votes(
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
