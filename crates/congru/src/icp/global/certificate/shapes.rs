use std::f64::consts::TAU;

use crate::{Error, Point, PointCloud, RigidTransform, Vector};

/// A wavy sheet with no symmetry, 20 cm across.
pub(super) fn sheet() -> Result<PointCloud, Error> {
    let points: Vec<Point> = (0..625)
        .map(|index| {
            let along = f64::from(index % 25) * 0.008;
            let across = f64::from(index / 25) * 0.008 * (1.0 + along);
            let height = 0.04 * (15.0 * along).sin() * (10.0 * across).cos() + 0.2 * along * along;
            Point::new(along, across, height)
        })
        .collect();
    PointCloud::new(points)
}

/// A flat ring with four lobes: a quarter turn leaves it as it was.
pub(super) fn clover() -> Result<PointCloud, Error> {
    let points: Vec<Point> = (0..240)
        .map(|step| {
            let angle = TAU * f64::from(step) / 240.0;
            let radius = 0.1 * (1.0 + 0.3 * (4.0 * angle).cos());
            Point::new(radius * angle.cos(), radius * angle.sin(), 0.0)
        })
        .collect();
    PointCloud::new(points)
}

pub(super) fn truth() -> RigidTransform {
    RigidTransform::from_axis_angle(Vector::new(1.1, -0.4, 0.7), [0.03, -0.02, 0.01])
}

/// `count` points of `target`, spread along it, in another frame.
pub(super) fn marks(target: &PointCloud, count: usize) -> Result<PointCloud, Error> {
    let step = target.count() / count;
    let moved: Vec<Point> = target
        .points()
        .iter()
        .step_by(step)
        .take(count)
        .map(|point| truth().inverse().apply(point))
        .collect();
    PointCloud::new(moved)
}
