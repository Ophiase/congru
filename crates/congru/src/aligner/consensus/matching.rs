use kiddo::{ImmutableKdTree, SquaredEuclidean};

use super::fpfh::{Descriptor, LENGTH};
use crate::{Error, Point};

/// Below this many mutual pairs, every pair is kept instead: a consensus
/// needs material more than purity.
const ENOUGH_MUTUAL: usize = 30;

/// A searchable set of descriptors, each tied to the point it describes.
#[derive(Debug)]
pub(crate) struct Described {
    pub points: Vec<Point>,
    pub features: Vec<Descriptor>,
    tree: ImmutableKdTree<f64, LENGTH>,
}

impl Described {
    pub(crate) fn new(points: Vec<Point>, features: Vec<Descriptor>) -> Result<Self, Error> {
        let rows: Vec<[f64; LENGTH]> = features.iter().map(|feature| (*feature).into()).collect();
        let tree = ImmutableKdTree::new_from_slice(&rows).map_err(|_| Error::Degenerate)?;
        Ok(Self {
            points,
            features,
            tree,
        })
    }

    fn closest(&self, feature: &Descriptor) -> usize {
        let row: [f64; LENGTH] = (*feature).into();
        self.tree
            .query(&row)
            .nearest_one::<SquaredEuclidean<f64>>()
            .execute()
            .item as usize
    }
}

/// Scan points paired with the model point whose descriptor is closest,
/// kept only when the model point's closest scan descriptor is that same
/// point (mutual nearest neighbours), which removes most wrong pairs on
/// smooth or repetitive surfaces.
pub(crate) fn pairs(scan: &Described, model: &Described) -> Vec<(Point, Point)> {
    let forward: Vec<usize> = scan
        .features
        .iter()
        .map(|feature| model.closest(feature))
        .collect();
    let mutual: Vec<(Point, Point)> = forward
        .iter()
        .enumerate()
        .filter(|(index, matched)| scan.closest(&model.features[**matched]) == *index)
        .map(|(index, matched)| (scan.points[index], model.points[*matched]))
        .collect();
    if mutual.len() >= ENOUGH_MUTUAL {
        return mutual;
    }
    forward
        .iter()
        .enumerate()
        .map(|(index, matched)| (scan.points[index], model.points[*matched]))
        .collect()
}
