use crate::fit::Match;
use crate::geometry::count_as_float;
use crate::search::map;
use crate::{Icp, Metric, PointCloud, RigidTransform, Target};

/// The kept, weighted pairs between the source moved by `pose` and the
/// target: nearest neighbours, minus those beyond the correspondence
/// distance, minus the farthest beyond the overlap.
pub(crate) fn correspond(
    source: &PointCloud,
    target: &Target,
    pose: &RigidTransform,
    icp: &Icp,
) -> Vec<Match> {
    let limit = icp
        .max_correspondence_distance
        .map_or(f64::INFINITY, |distance| distance.get().powi(2));
    let mut pairs: Vec<(f64, Match)> = map(source.points(), |point| {
        let moved = pose.apply(point);
        let nearest = target.index().nearest(&moved);
        let pair = Match {
            source: moved,
            target: target.cloud().points()[nearest.index],
            normal: target.normals().map(|normals| normals[nearest.index]),
            weight: 1.0,
        };
        (nearest.squared_distance, pair)
    })
    .into_iter()
    .filter(|(squared_distance, _)| *squared_distance <= limit)
    .collect();
    trim(&mut pairs, kept_count(source.count(), icp));
    pairs
        .into_iter()
        .map(|(_, pair)| Match {
            weight: icp.kernel.weight(residual(&pair, icp.metric)),
            ..pair
        })
        .collect()
}

pub(crate) fn residual(pair: &Match, metric: Metric) -> f64 {
    match (metric, pair.normal) {
        (Metric::PointToPlane, Some(normal)) => normal.dot(&(pair.source - pair.target)),
        _ => (pair.source - pair.target).norm(),
    }
}

fn trim(pairs: &mut Vec<(f64, Match)>, keep: usize) {
    if keep < pairs.len() {
        pairs.select_nth_unstable_by(keep, |left, right| left.0.total_cmp(&right.0));
        pairs.truncate(keep);
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a non-negative count no larger than the source"
)]
fn kept_count(source_count: usize, icp: &Icp) -> usize {
    (count_as_float(source_count) * icp.overlap.get()).ceil() as usize
}
