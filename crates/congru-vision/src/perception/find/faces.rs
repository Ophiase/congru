use congru::{NearestNeighbors, Point, RigidTransform, Vector};
use nalgebra::{Matrix3, Rotation3, UnitQuaternion};

use super::pairs::Oriented;
use crate::pixel::coordinate;

/// Normals of one face agree within this cosine.
const SAME_DIRECTION: f64 = 0.9;
/// Fewer sampled points than this make no face.
const LEAST_POINTS: usize = 10;
/// A patch of the image may exceed a face by this many steps, for the noise.
const LENGTH_SLACK: f64 = 1.5;

/// A patch must span this share of a face's long side to be tried on it: a scrap
/// of wall says nothing of where a large face goes.
const LEAST_SPAN: f64 = 0.5;

/// Neighbours of a region lie within this share of the step of its plane: the
/// noise is close to half a step.
const PLANE_SHARE: f64 = 0.8;

/// A flat piece of the model or of the frame; axes from its spread.
#[derive(Debug, Clone, Copy)]
pub(super) struct Patch {
    centre: Point,
    normal: Vector,
    along: Vector,
    across: Vector,
    half_along: f64,
    half_across: f64,
}

impl Patch {
    fn of(points: &[Oriented]) -> Option<Self> {
        if points.len() < LEAST_POINTS {
            return None;
        }
        let count = coordinate(points.len());
        let centre = points
            .iter()
            .fold(Vector::zeros(), |sum, oriented| sum + oriented.point.coords)
            / count;
        let normal = (points
            .iter()
            .fold(Vector::zeros(), |sum, oriented| sum + oriented.normal)
            / count)
            .try_normalize(f64::EPSILON)?;
        let scatter = points.iter().fold(Matrix3::zeros(), |sum, oriented| {
            let offset = oriented.point.coords - centre;
            let flat = offset - normal * normal.dot(&offset);
            sum + flat * flat.transpose()
        });
        let eigen = scatter.symmetric_eigen();
        let along = eigen
            .eigenvectors
            .column(eigen.eigenvalues.imax())
            .into_owned();
        let across = normal.cross(&along);
        let reach = |axis: &Vector| {
            points
                .iter()
                .map(|oriented| axis.dot(&(oriented.point.coords - centre)).abs())
                .fold(0.0, f64::max)
        };
        Some(Self {
            centre: Point::from(centre),
            normal,
            half_along: reach(&along),
            half_across: reach(&across),
            along,
            across,
        })
    }

    /// Whether this patch of a frame can be part of `face`, in either
    /// direction of the face's sides.
    fn fits_in(&self, face: &Self, step: f64) -> bool {
        let slack = LENGTH_SLACK * step;
        if self.half_along.max(self.half_across) < LEAST_SPAN * face.half_along {
            return false;
        }
        self.half_along <= face.half_along + slack
            && self.half_across <= face.half_along + slack
            && self.half_across.min(self.half_along) <= face.half_across + slack
    }
}

/// The flat faces of a model, from its oriented samples: same direction, same
/// height along it.
pub(super) fn model_faces(points: &[Oriented], step: f64) -> Vec<Patch> {
    let mut groups: Vec<Vec<Oriented>> = Vec::new();
    for oriented in points {
        let home = groups.iter_mut().find(|group| {
            let first = group[0];
            first.normal.dot(&oriented.normal) >= SAME_DIRECTION
                && first.normal.dot(&(oriented.point - first.point)).abs() <= step / 2.0
        });
        match home {
            Some(group) => group.push(*oriented),
            None => groups.push(vec![*oriented]),
        }
    }
    groups.iter().filter_map(|group| Patch::of(group)).collect()
}

/// The flat regions of a frame: its points grown over neighbours of the same
/// direction, on the same plane.
pub(super) fn frame_patches(frame: &[Oriented], index: &NearestNeighbors, step: f64) -> Vec<Patch> {
    let mut taken = vec![false; frame.len()];
    let mut patches = Vec::new();
    for seed in 0..frame.len() {
        if taken[seed] {
            continue;
        }
        taken[seed] = true;
        let mut region = vec![seed];
        let mut next = 0;
        while next < region.len() {
            let current = frame[region[next]];
            next += 1;
            for neighbor in index.within(&current.point, 2.0 * step) {
                let other = frame[neighbor.index];
                let flat = frame[seed].normal.dot(&other.normal) >= SAME_DIRECTION
                    && current.normal.dot(&(other.point - current.point)).abs()
                        <= step * PLANE_SHARE;
                if flat && !taken[neighbor.index] {
                    taken[neighbor.index] = true;
                    region.push(neighbor.index);
                }
            }
        }
        let members: Vec<Oriented> = region.iter().map(|member| frame[*member]).collect();
        patches.extend(Patch::of(&members));
    }
    patches
}

/// Every pose laying `face` on `patch`, its outside to the camera: the four ways
/// its sides can match, and, when the patch is smaller, sitting at each end.
pub(super) fn placements(patch: &Patch, face: &Patch, step: f64) -> Vec<RigidTransform> {
    if !patch.fits_in(face, step) {
        return Vec::new();
    }
    [
        (
            patch.along,
            patch.across,
            patch.half_along,
            patch.half_across,
        ),
        (
            -patch.along,
            -patch.across,
            patch.half_along,
            patch.half_across,
        ),
        (
            patch.across,
            -patch.along,
            patch.half_across,
            patch.half_along,
        ),
        (
            -patch.across,
            patch.along,
            patch.half_across,
            patch.half_along,
        ),
    ]
    .into_iter()
    .flat_map(|sides| laid(patch, face, sides))
    .collect()
}

/// The poses for one way of matching the sides.
fn laid(
    patch: &Patch,
    face: &Patch,
    (along, across, half_along, half_across): (Vector, Vector, f64, f64),
) -> Vec<RigidTransform> {
    let target = Matrix3::from_columns(&[along, across, patch.normal]);
    let source = Matrix3::from_columns(&[face.along, face.across, face.normal]);
    let turn = UnitQuaternion::from_rotation_matrix(&Rotation3::from_matrix(
        &(target * source.transpose()),
    ));
    let room_along = (face.half_along - half_along).max(0.0);
    let room_across = (face.half_across - half_across).max(0.0);
    let mut poses = Vec::new();
    for slide_along in [-room_along, 0.0, room_along] {
        for slide_across in [-room_across, 0.0, room_across] {
            let centre = turn * face.centre.coords + along * slide_along + across * slide_across;
            let shift = patch.centre.coords - centre;
            poses.push(RigidTransform::from_parts(
                turn,
                [shift.x, shift.y, shift.z],
            ));
        }
    }
    poses
}
