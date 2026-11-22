use super::sample::Draws;
use crate::fit::{Match, kabsch};
use crate::{Point, RigidTransform};

/// Two side lengths of a sampled triangle agree when their ratio is at least
/// this: a cheap test that rejects most wrong triples before any fit.
const EDGE_AGREEMENT: f64 = 0.9;

/// The motions explaining the most pairs: fit on random triples of pairs,
/// count the pairs each brings within `tolerance`, keep the `keep` best,
/// most supported first.
pub(crate) fn consensus(
    pairs: &[(Point, Point)],
    tolerance: f64,
    trials: usize,
    seed: u64,
    keep: usize,
) -> Vec<(RigidTransform, usize)> {
    let mut best: Vec<(RigidTransform, usize)> = Vec::new();
    if pairs.len() < 3 {
        return best;
    }
    let mut draws = Draws::new(seed);
    let limit = tolerance * tolerance;
    for _ in 0..trials {
        let triple = [(); 3].map(|()| pairs[draws.below(pairs.len())]);
        if !consistent(&triple) {
            continue;
        }
        let matches = triple.map(|(source, target)| Match {
            source,
            target,
            normal: None,
            weight: 1.0,
        });
        let Ok(motion) = kabsch(&matches) else {
            continue;
        };
        let support = pairs
            .iter()
            .filter(|(source, target)| (motion.apply(source) - target).norm_squared() <= limit)
            .count();
        if best.len() < keep || best.last().is_some_and(|(_, weakest)| support > *weakest) {
            best.push((motion, support));
            best.sort_by_key(|(_, support)| std::cmp::Reverse(*support));
            best.truncate(keep);
        }
    }
    best
}

fn consistent(triple: &[(Point, Point); 3]) -> bool {
    [(0, 1), (1, 2), (2, 0)].iter().all(|&(first, second)| {
        let source = (triple[first].0 - triple[second].0).norm();
        let target = (triple[first].1 - triple[second].1).norm();
        let (shorter, longer) = if source < target {
            (source, target)
        } else {
            (target, source)
        };
        longer > 0.0 && shorter / longer >= EDGE_AGREEMENT
    })
}
