use crate::geometry::count_as_float;
use crate::search::map;
use crate::{PointCloud, RigidTransform, Target};

/// Share of the bounding-box diagonal under which a point counts as
/// explained when the caller gave no correspondence distance.
const DEFAULT_SHARE_OF_DIAGONAL: f64 = 0.05;

/// Mean truncated squared distance from the moved source to the target:
/// low when most points sit close, insensitive to how far the rest are.
pub(crate) fn truncated_cost(
    source: &PointCloud,
    target: &Target,
    pose: &RigidTransform,
    threshold: f64,
) -> f64 {
    let ceiling = threshold.powi(2);
    let total: f64 = map(source.points(), |point| {
        target
            .index()
            .nearest(&pose.apply(point))
            .squared_distance
            .min(ceiling)
    })
    .iter()
    .sum();
    total / count_as_float(source.count())
}

pub(crate) fn default_threshold(target: &Target) -> f64 {
    let points = target.cloud().points();
    let (lowest, highest) = points.iter().fold(
        (points[0].coords, points[0].coords),
        |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
    );
    (highest - lowest).norm() * DEFAULT_SHARE_OF_DIAGONAL
}
