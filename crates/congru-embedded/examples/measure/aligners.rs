use std::time::Instant;

use congru::{Convergence, Distance, Kernel, Metric, Point, RigidTransform, Target};
use congru_embedded as embedded;
use congru_embedded::{KdTree, Real, Workspace};

use crate::plan::{GATE, ITERATIONS, KERNEL_SCALE, Setup};

/// One alignment against the truth; a refusal has no errors.
#[derive(Clone, Copy, Default)]
pub struct Trial {
    pub errors: Option<(f64, f64)>,
    pub seconds: f64,
    pub iterations: u32,
}

pub fn coordinates<Float: Real>(points: &[Point]) -> Vec<[Float; 3]> {
    points
        .iter()
        .map(|point| [point.x, point.y, point.z].map(Float::of))
        .collect()
}

pub fn main_trial(
    setup: &Setup,
    source: &[Point],
    target: &Target,
    truth: &RigidTransform,
    guess: &RigidTransform,
) -> anyhow::Result<Trial> {
    let icp = congru::Icp {
        metric: setup.metric,
        kernel: if setup.robust {
            Kernel::Cauchy(Distance::new(KERNEL_SCALE)?)
        } else {
            Kernel::Quadratic
        },
        max_correspondence_distance: Some(Distance::new(GATE)?),
        convergence: Convergence {
            max_iterations: ITERATIONS,
            ..Convergence::default()
        },
        ..congru::Icp::default()
    };
    let cloud = congru::PointCloud::new(source.to_vec())?;
    let started = Instant::now();
    let found = icp.align(&cloud, target, guess);
    let seconds = started.elapsed().as_secs_f64();
    Ok(found.map_or(
        Trial {
            seconds,
            ..Trial::default()
        },
        |found| Trial {
            errors: Some((
                found.transform.rotation_error_to(truth),
                found.transform.translation_error_to(truth),
            )),
            seconds,
            iterations: found.iterations,
        },
    ))
}

pub fn embedded_trial<Float: Real>(
    setup: &Setup,
    source: &[Point],
    target: &KdTree<Float>,
    truth: &RigidTransform,
    guess: &RigidTransform,
) -> Trial {
    let icp = embedded_icp(setup, target);
    let raw: Vec<[Float; 3]> = coordinates(source);
    let mut workspace = Workspace::new(raw.len());
    let start = pose_of::<Float>(guess);
    let started = Instant::now();
    let found = icp.align(&raw, target, &start, &mut workspace);
    let seconds = started.elapsed().as_secs_f64();
    found.map_or(
        Trial {
            seconds,
            ..Trial::default()
        },
        |found| {
            let truth = pose_of::<f64>(truth);
            let pose = embedded::Pose {
                rotation: embedded::UnitQuaternion::normalised(
                    found.pose.rotation.components().map(Real::as_f64),
                ),
                translation: embedded::Vector3(found.pose.translation.0.map(Real::as_f64)),
            };
            Trial {
                errors: Some((
                    pose.rotation_error_to(&truth),
                    pose.translation_error_to(&truth),
                )),
                seconds,
                iterations: found.iterations,
            }
        },
    )
}

fn embedded_icp<Float: Real>(setup: &Setup, target: &KdTree<Float>) -> embedded::Icp<Float> {
    embedded::Icp {
        metric: match setup.metric {
            Metric::PointToPlane => embedded::Metric::PointToPlane,
            Metric::PointToPoint => embedded::Metric::PointToPoint,
        },
        kernel: if setup.robust {
            embedded::Kernel::Cauchy(Float::of(KERNEL_SCALE))
        } else {
            embedded::Kernel::Quadratic
        },
        gate: Some(Float::of(GATE)),
        convergence: embedded::Convergence {
            max_iterations: ITERATIONS,
            ..embedded::Convergence::for_extent(target.extent())
        },
    }
}

fn pose_of<Float: Real>(transform: &RigidTransform) -> embedded::Pose<Float> {
    let rotation = transform.rotation();
    let translation = transform.translation();
    embedded::Pose {
        rotation: embedded::UnitQuaternion::normalised(
            [rotation.w, rotation.i, rotation.j, rotation.k].map(Float::of),
        ),
        translation: embedded::Vector3(
            [translation.x, translation.y, translation.z].map(Float::of),
        ),
    }
}
