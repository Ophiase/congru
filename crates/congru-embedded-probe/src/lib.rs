//! Uses every part of `congru-embedded` from a program with no standard
//! library, so that a link keeps exactly what firmware would. On a host it is
//! an ordinary program, which keeps the workspace's builds and lints working.

#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
mod bare;

#[cfg(target_os = "none")]
pub use bare::run;

use core::hint::black_box;

use congru_embedded::{
    Convergence, Error, Icp, KdTree, Kernel, Metric, Pair, Pose, Real, UnitQuaternion, Vector3,
    Workspace, kabsch, similarity,
};

const SIDE: usize = 12;
const NORMAL_PATCH: usize = 8;

/// A curved patch, moved a little, aligned back by each method. Inputs pass
/// through `black_box`: no call may fold away at compile time.
///
/// # Errors
/// What the library refuses, never on these points.
pub fn workout<Float: Real>() -> Result<Float, Error> {
    let mut grid = [[Float::ZERO; 3]; SIDE * SIDE];
    for (index, point) in grid.iter_mut().enumerate() {
        let across = Float::of_count(index % SIDE) * Float::of(0.1);
        let along = Float::of_count(index / SIDE) * Float::of(0.1);
        *point = black_box([across, along, across * along * (across - along)]);
    }
    let motion = Pose {
        rotation: UnitQuaternion::from_scaled_axis(black_box(Vector3([
            Float::of(0.03),
            Float::of(-0.02),
            Float::of(0.04),
        ]))),
        translation: Vector3([Float::of(0.01), Float::ZERO, Float::of(-0.01)]),
    };
    let moved = grid.map(|point| motion.apply(Vector3(point)).0);
    let pairs = grid.iter().zip(&moved).map(|(from, to)| Pair {
        source: Vector3(*from),
        target: Vector3(*to),
        weight: Float::ONE,
    });
    let fitted = kabsch(pairs.clone())?;
    let scaled = similarity(pairs)?;
    let mut tree = KdTree::new(&moved)?;
    tree.estimate_normals::<NORMAL_PATCH>()?;
    let mut workspace = Workspace::new(grid.len());
    let mut total = fitted.rotation.angle() + scaled.scale;
    for metric in [Metric::PointToPoint, Metric::PointToPlane] {
        let icp = Icp {
            metric,
            kernel: Kernel::Tukey(Float::of(0.2)),
            gate: Some(Float::of(0.3)),
            convergence: Convergence::for_extent(tree.extent()),
        };
        total += icp
            .align(&grid, &tree, &Pose::identity(), &mut workspace)?
            .rmse;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workout_is_accepted_in_both_scalars() {
        assert!(workout::<f32>().is_ok());
        assert!(workout::<f64>().is_ok());
    }
}
