use nalgebra::{Matrix6, Unit, Vector6};

use super::Match;
use super::plane::normal_equations;
use crate::geometry::{count_as_float, mean};
use crate::{Point, Vector};

const WEAK_SHARE: f64 = 1e-2;
const TRANSLATION_SHARE: f64 = 1e-3;
const MINIMUM_PAIRS: usize = 6;

/// How firmly the kept pairs pin each of the six degrees of freedom, read
/// from the point-to-plane normal equations at the final pose. Rotations are
/// measured by the arc they sweep at the source's radius, so a turn and a
/// slide compare in the same unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Observability {
    /// Smallest over largest stiffness: near 1 when every motion is pinned,
    /// near 0 when one is free.
    pub conditioning: f64,
    /// Motions pinned less than a hundredth as firmly as the stiffest one,
    /// weakest first. Empty when the pose is determined.
    pub weak_motions: Box<[WeakMotion]>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeakMotion {
    /// Its stiffness over the largest one, in `[0, 1)`.
    pub stiffness: f64,
    pub kind: MotionKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionKind {
    /// A turn about the line along `axis` through `through`, the point of
    /// that line closest to the source's centroid.
    Rotation {
        axis: Unit<Vector>,
        through: Point,
    },
    Translation {
        direction: Unit<Vector>,
    },
}

/// `None` when a pair lacks a normal or fewer than six pairs are kept.
pub(crate) fn observe(matches: &[Match]) -> Option<Observability> {
    if matches.len() < MINIMUM_PAIRS {
        return None;
    }
    let sources: Vec<Point> = matches.iter().map(|pair| pair.source).collect();
    let centre = mean(&sources);
    let radius = (sources
        .iter()
        .map(|point| (point - centre).norm_squared())
        .sum::<f64>()
        / count_as_float(sources.len()))
    .sqrt();
    if radius <= 0.0 {
        return None;
    }
    let (hessian, _) = normal_equations(matches, &centre).ok()?;
    let scale = Matrix6::from_diagonal(&Vector6::new(
        1.0 / radius,
        1.0 / radius,
        1.0 / radius,
        1.0,
        1.0,
        1.0,
    ));
    let eigen = (scale * hessian * scale).symmetric_eigen();
    let stiffest = eigen.eigenvalues.max();
    if stiffest <= 0.0 {
        return None;
    }
    let mut weak_motions: Vec<WeakMotion> = eigen
        .eigenvalues
        .iter()
        .zip(eigen.eigenvectors.column_iter())
        .filter(|(value, _)| **value < WEAK_SHARE * stiffest)
        .filter_map(|(value, vector)| {
            let rotation = vector.fixed_rows::<3>(0) / radius;
            let translation = vector.fixed_rows::<3>(3).into_owned();
            Some(WeakMotion {
                stiffness: (value / stiffest).max(0.0),
                kind: classify(rotation.into_owned(), translation, &centre, radius)?,
            })
        })
        .collect();
    weak_motions.sort_by(|weaker, stronger| weaker.stiffness.total_cmp(&stronger.stiffness));
    Some(Observability {
        conditioning: (eigen.eigenvalues.min() / stiffest).max(0.0),
        weak_motions: weak_motions.into_boxed_slice(),
    })
}

/// A small motion `p ↦ p + ω × (p − c) + t` is a pure slide when `ω` is
/// negligible; otherwise it is a turn about the axis where the motion of
/// the points vanishes, at `c + ω × t / |ω|²`.
fn classify(
    rotation: Vector,
    translation: Vector,
    centre: &Point,
    radius: f64,
) -> Option<MotionKind> {
    if rotation.norm() * radius < TRANSLATION_SHARE * translation.norm() {
        return Some(MotionKind::Translation {
            direction: Unit::try_new(translation, f64::EPSILON)?,
        });
    }
    Some(MotionKind::Rotation {
        axis: Unit::try_new(rotation, f64::EPSILON)?,
        through: centre + rotation.cross(&translation) / rotation.norm_squared(),
    })
}
