//! A model moving in front of a sensor, frame after frame, with the truth
//! kept at every frame: the test sets of tracking.

mod path;
mod visibility;

use nalgebra::Vector3;
use rand::SeedableRng;
use rand::rngs::ChaCha8Rng;
use rand_distr::{Distribution, Normal};
use recalage::{Error, Point, PointCloud, RigidTransform};

pub use path::{Drift, Jump};

use crate::shapes;
use visibility::visible;

const OCCLUDER_POINTS: usize = 1500;

/// A sequence, every effect off by default. The sensor sits at the origin
/// looking along +z; poses map the model's frame to the sensor's.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub frames: usize,
    pub drift: Drift,
    pub jumps: Vec<Jump>,
    /// Standard deviation of the noise on every observed point.
    pub noise: f64,
    /// Radius of a ball crossing between sensor and model, from left to
    /// right over the whole sequence; zero for none.
    pub occluder: f64,
    /// Radial distortion `k`: a point at `r` from the optical axis, in
    /// units of depth, is seen at `r (1 + k r²)`.
    pub distortion: f64,
}

/// One frame: what the sensor returned, the truth, and what happened.
#[derive(Debug, Clone, PartialEq)]
pub struct Shot {
    pub cloud: PointCloud,
    pub truth: RigidTransform,
    /// The pose jumped since the last frame.
    pub jumped: bool,
}

impl Sequence {
    /// `model` is a dense sampling of the model's surface.
    ///
    /// # Errors
    /// [`Error::Empty`] when a frame sees nothing.
    pub fn shoot(&self, model: &PointCloud, seed: u64) -> Result<Vec<Shot>, Error> {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let noise = Normal::new(0.0, self.noise).map_err(|_| Error::InvalidParameter {
            name: "noise",
            expected: "finite and non-negative",
            value: self.noise,
        })?;
        let ball = shapes::lumpy_sphere(OCCLUDER_POINTS)?;
        (0..self.frames)
            .map(|frame| {
                let truth = self.drift.pose(frame, &self.jumps);
                let mut scene: Vec<Point> = model
                    .points()
                    .iter()
                    .map(|point| truth.apply(point))
                    .collect();
                scene.extend(self.occluder_at(frame, &ball, &truth));
                let observed = visible(&scene)
                    .into_iter()
                    .map(|point| {
                        self.distort(&point) + Vector3::from_fn(|_, _| noise.sample(&mut random))
                    })
                    .collect::<Vec<_>>();
                Ok(Shot {
                    cloud: PointCloud::new(observed)?,
                    truth,
                    jumped: self.jumps.iter().any(|jump| jump.frame == frame),
                })
            })
            .collect()
    }

    fn occluder_at(&self, frame: usize, ball: &PointCloud, truth: &RigidTransform) -> Vec<Point> {
        if self.occluder <= 0.0 {
            return Vec::new();
        }
        let progress =
            crate::casts::as_float(frame) / crate::casts::as_float(self.frames.max(2) - 1);
        let target = truth.translation();
        let centre = Vector3::new(
            target.x + (progress - 0.5) * 6.0 * self.occluder,
            target.y,
            target.z * 0.6,
        );
        ball.points()
            .iter()
            .map(|point| Point::from(centre + point.coords * self.occluder))
            .collect()
    }

    fn distort(&self, point: &Point) -> Point {
        let squared = (point.x * point.x + point.y * point.y) / (point.z * point.z);
        let scale = 1.0 + self.distortion * squared;
        Point::new(point.x * scale, point.y * scale, point.z)
    }
}
