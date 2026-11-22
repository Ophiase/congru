mod fpfh;
mod matching;
mod ransac;
mod sample;

use std::num::NonZeroUsize;

use nalgebra::Unit;

use super::{Aligner, Estimate, Frame};
use crate::{
    Distance, Error, Icp, OrientedCloud, Point, PointCloud, RigidTransform, Target, Vector,
};
use fpfh::describe;
use matching::{Described, pairs};
use ransac::consensus;

const TRIALS: usize = 20_000;
/// The best supported motions, checked against the whole shape.
const CANDIDATES: usize = 20;
const SEED: u64 = 0x0c0f_fee0;
/// A pair belongs to the consensus within this many voxels.
const TOLERANCE_VOXELS: f64 = 2.0;
/// Neighbours a normal is fitted to; the histograms use a wider ring.
const NORMAL_NEIGHBORS: usize = 16;

/// A global method that does not start from rotations but from the shape:
/// both clouds are thinned on a voxel grid, each point gets a fast point
/// feature histogram, scan and model points whose histograms are mutual
/// nearest neighbours are paired, a RANSAC over triples of pairs finds the
/// motion most pairs agree on, and `refine` polishes it. Needs no guess,
/// and does not need the scan to cover most of the model.
#[derive(Debug)]
pub struct Consensus {
    pub name: String,
    /// Polishes the consensus; the frame's guess is never used.
    pub refine: Icp,
    voxel: Distance,
    neighbors: NonZeroUsize,
    model: Described,
    target: Target,
}

impl Consensus {
    /// `voxel` sets the scale: a few times the noise and about a thirtieth
    /// of the model; `neighbors` the ring a histogram gathers, 30 to 60.
    ///
    /// # Errors
    /// When the thinned model is smaller than a neighbourhood.
    pub fn new(
        name: impl Into<String>,
        model: &OrientedCloud,
        voxel: Distance,
        neighbors: usize,
        refine: Icp,
    ) -> Result<Self, Error> {
        let neighbors = NonZeroUsize::new(neighbors).ok_or(Error::InvalidParameter {
            name: "neighbors",
            expected: "at least 3",
            value: 0.0,
        })?;
        Ok(Self {
            name: name.into(),
            refine,
            voxel,
            neighbors,
            model: featured(model.cloud(), voxel, neighbors)?,
            target: Target::from_oriented(model.clone())?,
        })
    }
}

impl Aligner for Consensus {
    fn name(&self) -> &str {
        &self.name
    }

    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error> {
        let scan = featured(frame.scan, self.voxel, self.neighbors)?;
        let pairs = pairs(&scan, &self.model);
        let tolerance = TOLERANCE_VOXELS * self.voxel.get();
        let candidates = consensus(&pairs, tolerance, TRIALS, SEED, CANDIDATES);
        let motion = candidates
            .iter()
            .map(|(motion, _)| {
                (
                    overlap(&scan.points, &self.target, motion, tolerance),
                    *motion,
                )
            })
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, motion)| motion)
            .ok_or(Error::TooFewCorrespondences {
                found: pairs.len(),
                needed: 3,
            })?;
        Ok(Estimate::from_alignment(self.refine.align(
            frame.scan,
            &self.target,
            &motion,
        )?))
    }
}

/// Share of the thinned scan the motion brings within `tolerance` of the
/// model: a check on the whole shape, which the pairs alone cannot give.
fn overlap(points: &[Point], target: &Target, motion: &RigidTransform, tolerance: f64) -> f64 {
    let limit = tolerance * tolerance;
    let near = points
        .iter()
        .filter(|point| {
            target
                .index()
                .nearest(&motion.apply(point))
                .squared_distance
                <= limit
        })
        .count();
    crate::geometry::count_as_float(near) / crate::geometry::count_as_float(points.len().max(1))
}

/// The cloud thinned on the voxel grid, and a descriptor per kept point.
/// Histograms need normals of a consistent sign: each is turned away from
/// the centroid, the same rule for scan and model.
fn featured(
    cloud: &PointCloud,
    voxel: Distance,
    neighbors: NonZeroUsize,
) -> Result<Described, Error> {
    let thinned = cloud.voxel_downsample(voxel);
    let around = NORMAL_NEIGHBORS.min(thinned.count());
    let oriented = Target::with_estimated_normals(thinned, around)?;
    let centre = oriented.cloud().centroid();
    let outward: Vec<Unit<Vector>> = oriented
        .cloud()
        .points()
        .iter()
        .zip(oriented.normals().ok_or(Error::MissingNormals)?)
        .map(|(point, normal)| {
            if normal.dot(&(point - centre)) < 0.0 {
                -*normal
            } else {
                *normal
            }
        })
        .collect();
    let ring =
        NonZeroUsize::new(neighbors.get().min(oriented.cloud().count())).unwrap_or(neighbors);
    Described::new(
        oriented.cloud().points().to_vec(),
        describe(oriented.cloud(), &outward, ring),
    )
}
