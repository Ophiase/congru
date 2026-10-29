use std::f64::consts::PI;

use congru::{Error, NearestNeighbors, Point, PointCloud, RigidTransform};
use congru_synth::{Corruption, MotionBounds, Scenario};

const SPIRAL_POINTS: u32 = 16;
const SPIRAL_TURN: f64 = 1.6 * PI;
const SHIFT: [f64; 3] = [2.5, -1.0, 0.0];

/// A flat spiral: few enough points to see each one, and no symmetry, so
/// a single motion aligns it.
pub fn spiral() -> Result<PointCloud, Error> {
    PointCloud::new(
        (0..SPIRAL_POINTS)
            .map(|index| {
                let angle = SPIRAL_TURN * f64::from(index) / f64::from(SPIRAL_POINTS - 1);
                let radius = 1.0 + 0.45 * angle;
                Point::new(radius * angle.cos(), radius * angle.sin(), 0.0)
            })
            .collect::<Vec<_>>(),
    )
}

/// The spiral seen through `corruption`, kept in its plane and in place;
/// outliers come last.
pub fn sight(model: &PointCloud, corruption: &Corruption, seed: u64) -> Result<Scenario, Error> {
    let still = MotionBounds {
        max_angle: 0.0,
        max_translation: 0.0,
    };
    let scenario = Scenario::generate(model, corruption, &still, seed)?;
    let flat: Vec<Point> = scenario
        .observed
        .points()
        .iter()
        .map(|point| Point::new(point.x, point.y, 0.0))
        .collect();
    Ok(Scenario {
        observed: PointCloud::new(flat)?,
        ..scenario
    })
}

/// Turned by `degrees` in the plane, then shifted.
pub fn turn(cloud: &PointCloud, degrees: f64) -> PointCloud {
    cloud.transformed(&RigidTransform::from_axis_angle(
        nalgebra::Vector3::z() * degrees.to_radians(),
        SHIFT,
    ))
}

pub fn observe(
    model: &PointCloud,
    corruption: &Corruption,
    degrees: f64,
    seed: u64,
) -> Result<PointCloud, Error> {
    Ok(turn(&sight(model, corruption, seed)?.observed, degrees))
}

pub fn nearest(points: &[Point], model: &PointCloud) -> Result<Vec<Point>, Error> {
    let index = NearestNeighbors::new(model)?;
    Ok(points
        .iter()
        .map(|point| model.points()[index.nearest(point).index])
        .collect())
}

pub fn together(first: &PointCloud, second: &PointCloud) -> Vec<Point> {
    first
        .points()
        .iter()
        .chain(second.points())
        .copied()
        .collect()
}
