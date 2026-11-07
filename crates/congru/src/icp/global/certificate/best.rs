use super::cube::{Cube, Problem};
use crate::search::map;
use crate::{Point, PointCloud, RigidTransform, fit_rigid};

const POLISH_STEPS: usize = 6;
/// Cube centres polished per batch: one already closes what eight close, sixty-four
/// find nothing more.
const POLISH_STARTS: usize = 8;

/// The best pose evaluated so far: the upper bound.
pub(super) struct Best {
    pub(super) pose: RigidTransform,
    pub(super) value: f64,
}

impl Best {
    pub(super) fn of(problem: &Problem<'_>, seeds: &[RigidTransform]) -> Self {
        let mut best = Self {
            pose: RigidTransform::identity(),
            value: f64::INFINITY,
        };
        for seed in seeds {
            best.offer(*seed, problem.value(seed));
        }
        best
    }

    pub(super) fn offer(&mut self, pose: RigidTransform, value: f64) {
        if value < self.value {
            self.pose = pose;
            self.value = value;
        }
    }

    /// Polishes from the lowest centres of a batch, each new, rather than from the
    /// best pose again: the upper bound sets the size of the tree, and being a
    /// millimetre high can multiply it without limit.
    pub(super) fn polish_lowest(
        &mut self,
        problem: &Problem<'_>,
        target: &PointCloud,
        children: &[Cube],
    ) {
        let mut lowest: Vec<&Cube> = children
            .iter()
            .filter(|cube| cube.value.is_finite())
            .collect();
        lowest.sort_unstable_by(|left, right| left.value.total_cmp(&right.value));
        lowest.truncate(POLISH_STARTS);
        if let Some(first) = lowest.first() {
            self.offer(problem.pose(first), first.value);
        }
        let starts: Vec<RigidTransform> = lowest.iter().map(|cube| problem.pose(cube)).collect();
        self.polish_from(problem, target, &starts);
    }

    pub(super) fn polish_from(
        &mut self,
        problem: &Problem<'_>,
        target: &PointCloud,
        starts: &[RigidTransform],
    ) {
        for (pose, value) in map(starts, |start| polished(problem, target, *start)) {
            self.offer(pose, value);
        }
    }
}

/// A few closed-form steps on the pairs within the truncation. The value returned is
/// that of the pose returned, evaluated: only that may become an upper bound.
fn polished(
    problem: &Problem<'_>,
    target: &PointCloud,
    start: RigidTransform,
) -> (RigidTransform, f64) {
    let (mut pose, mut kept, mut value) = (start, start, f64::INFINITY);
    for _ in 0..POLISH_STEPS {
        let (from, to): (Vec<Point>, Vec<Point>) = problem
            .centred
            .iter()
            .filter_map(|point| {
                let given = Point::from(point.coords + problem.centre);
                let found = problem.index.nearest(&pose.apply(&given));
                (found.squared_distance < problem.truncation_squared)
                    .then(|| (given, target.points()[found.index]))
            })
            .unzip();
        let Ok(fit) = fit_rigid(&from, &to, None) else {
            break;
        };
        pose = fit.transform;
        let reached = problem.value(&pose);
        if reached < value {
            kept = pose;
            value = reached;
        }
    }
    (kept, value)
}
