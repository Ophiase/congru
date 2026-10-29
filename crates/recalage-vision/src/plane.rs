use recalage::{Point, Vector};

const TRIALS: usize = 200;

/// The points left once the largest plane is taken away, when that plane
/// spreads wider than `wider_than`: a table or a turntable is wider than
/// the part it carries, a face of the part is not. RANSAC over `TRIALS`
/// triples picked by a fixed stride, so the result is deterministic. Points
/// within `tolerance` of the plane belong to it.
#[must_use]
pub fn without_support(points: &[Point], tolerance: f64, wider_than: f64) -> Vec<Point> {
    let Some((normal, offset, _)) = largest_plane(points, tolerance) else {
        return points.to_vec();
    };
    let on_plane = |point: &&Point| (normal.dot(&point.coords) - offset).abs() <= tolerance;
    if spread(points.iter().filter(on_plane)) <= wider_than {
        return points.to_vec();
    }
    points
        .iter()
        .filter(|point| !on_plane(point))
        .copied()
        .collect()
}

fn spread<'points>(points: impl Iterator<Item = &'points Point>) -> f64 {
    let mut bounds: Option<(Vector, Vector)> = None;
    for point in points {
        bounds = Some(
            bounds.map_or((point.coords, point.coords), |(lowest, highest)| {
                (lowest.inf(&point.coords), highest.sup(&point.coords))
            }),
        );
    }
    bounds.map_or(0.0, |(lowest, highest)| (highest - lowest).norm())
}

fn largest_plane(points: &[Point], tolerance: f64) -> Option<(Vector, f64, usize)> {
    let count = points.len();
    if count < 3 {
        return None;
    }
    (0..TRIALS)
        .filter_map(|trial| {
            let pick =
                |salt: usize| points[(trial * 7919 + salt * 104_729 + trial * trial * 31) % count];
            let (first, second, third) = (pick(1), pick(2), pick(3));
            let normal = (second - first)
                .cross(&(third - first))
                .try_normalize(f64::EPSILON)?;
            let offset = normal.dot(&first.coords);
            let support = points
                .iter()
                .filter(|point| (normal.dot(&point.coords) - offset).abs() <= tolerance)
                .count();
            Some((normal, offset, support))
        })
        .max_by_key(|(_, _, support)| *support)
}
