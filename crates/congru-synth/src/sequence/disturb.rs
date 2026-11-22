use std::f64::consts::TAU;

use congru::{Point, RigidTransform};
use nalgebra::{UnitQuaternion, Vector3};
use rand::{Rng, RngExt};
use rand_distr::{Distribution, Normal};

use crate::casts::as_float;
use crate::motion::unit_vector;

const BEND_PERIOD_FRAMES: f64 = 40.0;

/// A jolt of the sensor every frame: a rotation about it of standard
/// deviation `angle` (radians) and a shift of standard deviation `shift`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shake {
    pub angle: f64,
    pub shift: f64,
}

impl Shake {
    pub(super) fn jolt(&self, random: &mut impl Rng) -> RigidTransform {
        let gauss = |deviation: f64, random: &mut dyn Rng| {
            Normal::new(0.0, deviation).map_or(0.0, |normal| normal.sample(random))
        };
        let turn = unit_vector(random).into_inner() * gauss(self.angle, random);
        let shift = unit_vector(random).into_inner() * gauss(self.shift, random);
        RigidTransform::from_parts(UnitQuaternion::from_scaled_axis(turn), shift)
    }
}

/// A smooth, invertible warp of the model at `frame`, swinging back and
/// forth: each coordinate grows with the square of another, so the warp has
/// no rotation in it to first order and no point moves more than about
/// `amplitude`: a part that sags and flexes a little.
pub(super) fn bend(point: &Point, amplitude: f64, span: f64, frame: usize) -> Point {
    let swing = (TAU * as_float(frame) / BEND_PERIOD_FRAMES).sin();
    let half = span / 2.0;
    let square = |coordinate: f64| (coordinate / half).powi(2);
    point + Vector3::new(square(point.y), square(point.z), square(point.x)) * (amplitude * swing)
}

/// Keeps each point with probability `share`.
pub(super) fn thin(points: Vec<Point>, share: f64, random: &mut impl Rng) -> Vec<Point> {
    points
        .into_iter()
        .filter(|_| random.random::<f64>() < share)
        .collect()
}

/// `count` points uniform in the cube of side `side` around `centre`.
pub(super) fn clutter(
    centre: &Vector3<f64>,
    side: f64,
    count: usize,
    random: &mut impl Rng,
) -> Vec<Point> {
    (0..count)
        .map(|_| {
            Point::from(centre + Vector3::from_fn(|_, _| (random.random::<f64>() - 0.5) * side))
        })
        .collect()
}
