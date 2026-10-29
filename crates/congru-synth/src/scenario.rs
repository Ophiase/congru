use congru::{Error, NearestNeighbors, Point, PointCloud, RigidTransform};
use nalgebra::Vector3;
use rand::rngs::ChaCha8Rng;
use rand::{Rng, RngExt, SeedableRng};
use rand_distr::{Distribution, Normal, Poisson};

use crate::casts::{as_count, as_float};
use crate::motion::unit_vector;
use crate::{Corruption, MotionBounds, Outliers, Resampling};

/// Room around the model's bounding box where outliers may fall, as a share
/// of its size.
const OUTLIER_MARGIN: f64 = 0.1;

/// A corrupted, moved copy of a model, and the motion that moved it.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    /// What the sensor saw: `truth` applied to the corrupted model.
    pub observed: PointCloud,
    /// The motion from the model's frame to the observation's.
    pub truth: RigidTransform,
    /// How many of the observed points are outliers; they come last.
    pub outliers: usize,
}

impl Scenario {
    /// The same `seed` always gives the same scenario, on every platform.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] for a negative noise or Poisson mean;
    /// [`Error::Empty`] when nothing survives the crop and resampling.
    pub fn generate(
        model: &PointCloud,
        corruption: &Corruption,
        motion: &MotionBounds,
        seed: u64,
    ) -> Result<Self, Error> {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let visible = crop(model.points(), corruption.visible.get(), &mut random);
        let mut points = resample(&visible, corruption, &mut random)?;
        let outliers = scatter(model, corruption.outliers, &mut random)?;
        let outlier_count = outliers.len();
        points.extend(outliers);
        let truth = motion.draw(&mut random);
        Ok(Self {
            observed: PointCloud::new(points)?.transformed(&truth),
            truth,
            outliers: outlier_count,
        })
    }

    /// The motion an aligner should find to bring the observation back onto
    /// the model.
    #[must_use]
    pub fn expected(&self) -> RigidTransform {
        self.truth.inverse()
    }
}

fn crop(points: &[Point], share: f64, random: &mut impl Rng) -> Vec<Point> {
    let direction = unit_vector(random);
    let mut heights: Vec<(f64, Point)> = points
        .iter()
        .map(|point| (direction.dot(&point.coords), *point))
        .collect();
    heights.sort_by(|left, right| left.0.total_cmp(&right.0));
    let keep = as_count((as_float(heights.len()) * share).ceil());
    heights
        .into_iter()
        .take(keep)
        .map(|(_, point)| point)
        .collect()
}

fn resample(
    points: &[Point],
    corruption: &Corruption,
    random: &mut impl Rng,
) -> Result<Vec<Point>, Error> {
    let noise =
        Normal::new(0.0, corruption.noise).map_err(|_| invalid("noise", corruption.noise))?;
    let poisson = match corruption.resampling {
        Resampling::Poisson(mean) => {
            Some(Poisson::new(mean).map_err(|_| invalid("poisson mean", mean))?)
        }
        _ => None,
    };
    let mut observed = Vec::with_capacity(points.len());
    for point in points {
        let copies = match (corruption.resampling, &poisson) {
            (Resampling::One, _) => 1,
            (Resampling::Fixed(count), _) => count as usize,
            (Resampling::Poisson(_), Some(poisson)) => as_count(poisson.sample(random)),
            (Resampling::Poisson(_), None) => 0,
        };
        for _ in 0..copies {
            observed.push(point + Vector3::from_fn(|_, _| noise.sample(random)));
        }
    }
    Ok(observed)
}

fn scatter(
    model: &PointCloud,
    outliers: Outliers,
    random: &mut impl Rng,
) -> Result<Vec<Point>, Error> {
    let (lowest, highest) = grown_bounds(model.points());
    let mut draw = || {
        Point::from(Vector3::from_fn(|row, _| {
            random.random_range(lowest[row]..=highest[row])
        }))
    };
    match outliers {
        Outliers::None => Ok(Vec::new()),
        Outliers::InBox { count } => Ok((0..count).map(|_| draw()).collect()),
        Outliers::Exterior { count, clearance } => {
            let index = NearestNeighbors::new(model)?;
            let limit = clearance.get().powi(2);
            Ok(std::iter::repeat_with(draw)
                .filter(|point| index.nearest(point).squared_distance >= limit)
                .take(count)
                .collect())
        }
    }
}

fn grown_bounds(points: &[Point]) -> (Vector3<f64>, Vector3<f64>) {
    let (lowest, highest) = points.iter().fold(
        (points[0].coords, points[0].coords),
        |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
    );
    let margin = (highest - lowest) * OUTLIER_MARGIN;
    (lowest - margin, highest + margin)
}

const fn invalid(name: &'static str, value: f64) -> Error {
    Error::InvalidParameter {
        name,
        expected: "finite and non-negative",
        value,
    }
}
