//! A model moving in front of a sensor, frame after frame, with the truth
//! kept at every frame: the test sets of tracking.

mod disturb;
mod handheld;
mod path;

use congru::{Error, Point, PointCloud, RigidTransform};
use nalgebra::Vector3;
use rand::SeedableRng;
use rand::rngs::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

pub use disturb::Shake;
pub use handheld::Handheld;
pub use path::{Drift, Jump};

use crate::shapes;
use congru_vision::visible_from_origin;

const OCCLUDER_POINTS: usize = 1500;
const CLUTTER_SIDE: f64 = 0.35;

/// A sequence. The sensor sits at the origin looking along +z; poses map
/// the model's frame to the sensor's. Every effect is off in
/// [`Sequence::still`] and is turned up by setting its field.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub frames: usize,
    pub drift: Drift,
    pub jumps: Vec<Jump>,
    pub shake: Shake,
    pub handheld: Handheld,
    /// Largest displacement of the part's own flexing; zero for a rigid part.
    pub bend: f64,
    /// Share of the visible surface the sensor returns.
    pub density: f64,
    /// Standard deviation of the noise on every observed point.
    pub noise: f64,
    /// Points scattered around the part, every frame.
    pub clutter: usize,
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
    #[must_use]
    pub fn still(frames: usize, drift: Drift) -> Self {
        Self {
            frames,
            drift,
            jumps: Vec::new(),
            shake: Shake::default(),
            handheld: Handheld::default(),
            bend: 0.0,
            density: 1.0,
            noise: 0.0,
            clutter: 0,
            occluder: 0.0,
            distortion: 0.0,
        }
    }

    /// `model` is a dense sampling of the model's surface, `span` its size.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] for a negative noise; [`Error::Empty`]
    /// when a frame sees nothing.
    pub fn shoot(&self, model: &PointCloud, span: f64, seed: u64) -> Result<Vec<Shot>, Error> {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let noise = Normal::new(0.0, self.noise).map_err(|_| Error::InvalidParameter {
            name: "noise",
            expected: "finite and non-negative",
            value: self.noise,
        })?;
        let ball = shapes::lumpy_sphere(OCCLUDER_POINTS)?;
        (0..self.frames)
            .map(|frame| {
                let truth = self
                    .drift
                    .pose(frame, &self.jumps)
                    .then(&self.handheld.pose(frame))
                    .then(&self.shake.jolt(&mut random));
                let observed = self.observe(frame, model, span, &ball, &truth, &mut random);
                let observed: Vec<Point> = observed
                    .iter()
                    .map(|point| {
                        self.distort(point) + Vector3::from_fn(|_, _| noise.sample(&mut random))
                    })
                    .collect();
                Ok(Shot {
                    cloud: PointCloud::new(observed)?,
                    truth,
                    jumped: self.jumps.iter().any(|jump| jump.frame == frame),
                })
            })
            .collect()
    }

    /// What the sensor can see of the flexed model at `truth`, thinned, with
    /// clutter, before noise and distortion.
    fn observe(
        &self,
        frame: usize,
        model: &PointCloud,
        span: f64,
        ball: &PointCloud,
        truth: &RigidTransform,
        random: &mut ChaCha8Rng,
    ) -> Vec<Point> {
        let mut scene: Vec<Point> = model
            .points()
            .iter()
            .map(|point| truth.apply(&disturb::bend(point, self.bend, span, frame)))
            .collect();
        scene.extend(self.occluder_at(frame, ball, truth));
        let facing = visible_from_origin(&scene)
            .into_iter()
            .map(|index| scene[index])
            .collect();
        let mut observed = disturb::thin(facing, self.density, random);
        observed.extend(disturb::clutter(
            &truth.translation(),
            CLUTTER_SIDE * span / 0.2,
            self.clutter,
            random,
        ));
        observed
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
            target.z * 0.75,
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
