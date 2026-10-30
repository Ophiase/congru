use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use congru::{Point, RigidTransform, Vector};

/// Angles are read to a thirtieth of a half turn, as in the original paper.
pub(super) const ANGLE_BINS: usize = 30;

/// A pair's distance in steps and its three angles in bins.
pub(super) type Key = [usize; 4];

/// An oriented point, as the pairs need it.
#[derive(Debug, Clone, Copy)]
pub(super) struct Oriented {
    pub point: Point,
    pub normal: Vector,
}

/// Every pair of oriented model points, filed by its key: for each, the
/// first point and the angle of the second about the first's normal.
#[derive(Debug)]
pub(super) struct PairTable {
    pub step: f64,
    pub entries: HashMap<Key, Vec<(usize, f64)>>,
}

impl PairTable {
    pub(super) fn new(model: &[Oriented], step: f64) -> Self {
        let mut entries: HashMap<Key, Vec<(usize, f64)>> = HashMap::new();
        for (first, reference) in model.iter().enumerate() {
            let frame = local_frame(reference);
            for (second, other) in model.iter().enumerate() {
                if first != second {
                    let key = key(reference, other, step);
                    entries
                        .entry(key)
                        .or_default()
                        .push((first, turn(&frame, &other.point)));
                }
            }
        }
        Self { step, entries }
    }
}

/// The pair feature of Drost et al.: the distance, the angle of each normal
/// to the line joining the points, and the angle between the normals.
pub(super) fn key(first: &Oriented, second: &Oriented, step: f64) -> Key {
    let offset = second.point - first.point;
    let length = offset.norm().max(f64::EPSILON);
    let direction = offset / length;
    [
        whole_steps(length / step),
        angle_bin(first.normal.dot(&direction)),
        angle_bin(second.normal.dot(&direction)),
        angle_bin(first.normal.dot(&second.normal)),
    ]
}

/// The motion that brings `reference` to the origin and its normal onto
/// the x axis.
pub(super) fn local_frame(reference: &Oriented) -> RigidTransform {
    let axis = reference.normal.cross(&Vector::x());
    let rotation = if axis.norm() > 1e-9 {
        axis.normalize() * reference.normal.dot(&Vector::x()).clamp(-1.0, 1.0).acos()
    } else if reference.normal.x > 0.0 {
        Vector::zeros()
    } else {
        Vector::y() * PI
    };
    RigidTransform::from_axis_angle(Vector::zeros(), -reference.point.coords)
        .then(&RigidTransform::from_axis_angle(rotation, [0.0; 3]))
}

/// The angle about the x axis of `point` once in the local `frame`.
pub(super) fn turn(frame: &RigidTransform, point: &Point) -> f64 {
    let local = frame.apply(point);
    local.z.atan2(local.y)
}

/// The bin of an angle about the normal, in a whole turn.
pub(super) fn turn_bin(angle: f64) -> usize {
    whole_steps(angle.rem_euclid(TAU) / TAU * bins()).min(ANGLE_BINS - 1)
}

/// The angle at the centre of a turn bin.
pub(super) fn turn_of(bin: usize) -> f64 {
    (as_float(bin) + 0.5) / bins() * TAU
}

fn angle_bin(cosine: f64) -> usize {
    whole_steps(cosine.clamp(-1.0, 1.0).acos() / PI * bins()).min(ANGLE_BINS - 1)
}

fn bins() -> f64 {
    as_float(ANGLE_BINS)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a finite, non-negative ratio, floored to a bin"
)]
fn whole_steps(ratio: f64) -> usize {
    ratio.floor().max(0.0) as usize
}

#[expect(clippy::cast_precision_loss, reason = "a bin index, far below 2^52")]
const fn as_float(count: usize) -> f64 {
    count as f64
}
