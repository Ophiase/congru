use std::cmp::Ordering;

use crate::geometry::count_as_float;
use crate::{MeshIndex, NearestNeighbors, Point, PointCloud, RigidTransform, Surface, Vector};

/// Wider than rounding, as a share of the source's reach: a bound that errs must
/// err low.
const SLACK: f64 = 1e-9;

/// What a source is aligned on: the proof is about this, samples or the surface.
#[derive(Clone, Copy)]
pub(super) enum Shape<'data> {
    Points {
        index: &'data NearestNeighbors,
        cloud: &'data PointCloud,
    },
    Mesh(&'data MeshIndex),
}

impl Shape<'_> {
    pub(super) fn squared(&self, query: &Point) -> f64 {
        match self {
            Self::Points { index, .. } => index.nearest(query).squared_distance,
            Self::Mesh(mesh) => mesh
                .closest(query)
                .map_or(f64::INFINITY, |closest| closest.squared_distance),
        }
    }

    /// The nearest point, and the square of how far it is.
    pub(super) fn nearest(&self, query: &Point) -> Option<(f64, Point)> {
        match self {
            Self::Points { index, cloud } => {
                let found = index.nearest(query);
                Some((found.squared_distance, cloud.points()[found.index]))
            }
            Self::Mesh(mesh) => mesh
                .closest(query)
                .map(|closest| (closest.squared_distance, closest.point)),
        }
    }
}

/// The source centred on its centroid, so that a turn moves it least, and what a
/// pose costs on the target.
pub(super) struct Problem<'data> {
    pub(super) centred: Vec<Point>,
    pub(super) centre: Vector,
    pub(super) reach: f64,
    pub(super) shape: Shape<'data>,
    pub(super) truncation_squared: f64,
}

impl Problem<'_> {
    /// The value of a pose given on the centred source.
    fn cost(&self, rotation: &nalgebra::UnitQuaternion<f64>, translation: &Vector) -> f64 {
        let total: f64 = self
            .centred
            .iter()
            .map(|point| {
                let moved = rotation * point + translation;
                self.shape.squared(&moved).min(self.truncation_squared)
            })
            .sum();
        total / count_as_float(self.centred.len())
    }

    /// The value of a pose of the source as given.
    pub(super) fn value(&self, pose: &RigidTransform) -> f64 {
        let rotation = pose.rotation();
        self.cost(&rotation, &(pose.translation() + rotation * self.centre))
    }

    /// Cube `cube` with its bound and the value at its centre, Hartley and Kahl's
    /// radius for what a turn can move a point. Past `ceiling` the sum stops, still a
    /// valid bound as every term is positive, and the value is left infinite.
    pub(super) fn measured(&self, cube: Cube, ceiling: f64) -> Cube {
        let rotation = nalgebra::UnitQuaternion::from_scaled_axis(cube.rotation);
        let turn = turn_share(cube.half_rotation);
        let shift = 3.0_f64.sqrt() * cube.half_translation;
        let slack = SLACK * self.reach;
        let count = count_as_float(self.centred.len());
        let last = ceiling * count;
        let (mut lower, mut value) = (0.0, 0.0);
        for point in &self.centred {
            if lower > last {
                return Cube {
                    lower: lower / count,
                    value: f64::INFINITY,
                    ..cube
                };
            }
            let moved = rotation * point + cube.translation;
            let squared = self.shape.squared(&moved);
            value += squared.min(self.truncation_squared);
            let radius = turn * point.coords.norm() + shift + slack;
            let closest = (squared.sqrt() - radius).max(0.0);
            lower += (closest * closest).min(self.truncation_squared);
        }
        Cube {
            lower: lower / count,
            value: value / count,
            ..cube
        }
    }

    /// The pose of the source as given, at the centre of `cube`.
    pub(super) fn pose(&self, cube: &Cube) -> RigidTransform {
        let rotation = nalgebra::UnitQuaternion::from_scaled_axis(cube.rotation);
        RigidTransform::from_parts(rotation, cube.translation - rotation * self.centre)
    }
}

/// How far a turn inside a cube of half-side `half` moves a unit vector.
fn turn_share(half: f64) -> f64 {
    let angle = 3.0_f64.sqrt() * half / 2.0;
    2.0 * angle.min(std::f64::consts::FRAC_PI_2).sin()
}

/// A cube of rotation vectors times a cube of translations, each by half-side.
#[derive(Debug, Clone, Copy)]
pub(super) struct Cube {
    pub(super) rotation: Vector,
    pub(super) translation: Vector,
    pub(super) half_rotation: f64,
    pub(super) half_translation: f64,
    pub(super) lower: f64,
    pub(super) value: f64,
}

impl Cube {
    /// The eight halves along whichever of rotation and translation still moves the
    /// source most, among those coarser than their `resolution`. `None` when neither
    /// is.
    pub(super) fn split(&self, reach: f64, resolution: [f64; 2]) -> Option<[Self; 8]> {
        let turnable = self.half_rotation > resolution[0];
        let shiftable = self.half_translation > resolution[1];
        let turn_moves = turn_share(self.half_rotation) * reach;
        let shift_moves = 3.0_f64.sqrt() * self.half_translation;
        let along_rotation = match (turnable, shiftable) {
            (false, false) => return None,
            (true, false) => true,
            (false, true) => false,
            (true, true) => turn_moves >= shift_moves,
        };
        Some(std::array::from_fn(|corner| {
            self.child(corner, along_rotation)
        }))
    }

    /// Whether a rotation vector of this cube is at most a half turn: every rotation
    /// has one there, so a cube wholly past it only repeats others.
    pub(super) fn inside_the_turns(&self) -> bool {
        let nearest = self
            .rotation
            .map(|along| (along.abs() - self.half_rotation).max(0.0));
        nearest.norm() <= std::f64::consts::PI * (1.0 + SLACK)
    }

    fn child(&self, corner: usize, along_rotation: bool) -> Self {
        let half = if along_rotation {
            self.half_rotation
        } else {
            self.half_translation
        } / 2.0;
        let sign = |bit: usize| if corner >> bit & 1 == 0 { -half } else { half };
        let offset = Vector::new(sign(0), sign(1), sign(2));
        if along_rotation {
            Self {
                rotation: self.rotation + offset,
                half_rotation: half,
                ..*self
            }
        } else {
            Self {
                translation: self.translation + offset,
                half_translation: half,
                ..*self
            }
        }
    }
}

impl PartialEq for Cube {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Cube {}

impl PartialOrd for Cube {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The lowest bound first, in a heap that pops the greatest.
impl Ord for Cube {
    fn cmp(&self, other: &Self) -> Ordering {
        other.lower.total_cmp(&self.lower)
    }
}
