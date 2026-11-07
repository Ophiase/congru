mod cube;
#[cfg(test)]
mod shapes;
#[cfg(test)]
mod tests;

use std::collections::BinaryHeap;

use cube::{Cube, Problem};

use crate::search::map;
use crate::{
    Distance, Error, NearestNeighbors, Point, PointCloud, RigidTransform, Vector, fit_rigid,
};

/// Cubes split at once between two looks at the upper bound.
const BATCH: usize = 512;
const POLISH_STEPS: usize = 6;

/// A search with a proof: [branch and bound] over every rotation and a box of
/// translations, for the mean of squared distances to the nearest target point,
/// each truncated.
///
/// [branch and bound]: https://en.wikipedia.org/wiki/Branch_and_bound
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Certifier {
    /// Past this distance a point counts as an outlier, whatever how far.
    pub truncation: Distance,
    /// How far the source's centroid may land from the target's along each axis:
    /// nothing is claimed outside.
    pub reach: Distance,
    /// A cube is dropped once its bound passes the best value by this. Negative, it
    /// asks for a gap of that size; positive, it lists every pose this close.
    pub margin: f64,
    /// Half-sides below which a cube is not split: radians, then the unit of the data.
    pub resolution: [f64; 2],
    /// How many cubes to measure before stopping with the bound still open.
    pub budget: usize,
}

/// What a run proved. Within the domain, no pose costs less than `lower`.
#[derive(Debug, Clone, PartialEq)]
pub struct Certificate {
    pub lower: f64,
    /// The value of `pose`, the best seen.
    pub upper: f64,
    pub pose: RigidTransform,
    /// `false` when the budget ran out first: `lower` holds, and may be far.
    pub closed: bool,
    pub measured: usize,
    /// Poses the bound could not exclude at the resolution asked: several far apart
    /// say that the data does not choose.
    pub survivors: Vec<RigidTransform>,
}

impl Certificate {
    /// How much better than `value` any pose in the domain can be.
    #[must_use]
    pub fn gap(&self, value: f64) -> f64 {
        (value - self.lower).max(0.0)
    }
}

impl Certifier {
    /// `seeds` start the upper bound: the poses a method found, so that they are
    /// graded.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] for a resolution that is not positive or for more
    /// target points than an index holds.
    pub fn run(
        &self,
        source: &PointCloud,
        target: &PointCloud,
        seeds: &[RigidTransform],
    ) -> Result<Certificate, Error> {
        if self.resolution.iter().any(|side| *side <= 0.0) {
            return Err(Error::InvalidParameter {
                name: "resolution",
                expected: "positive half-sides",
                value: self.resolution[0].min(self.resolution[1]),
            });
        }
        let index = NearestNeighbors::new(target)?;
        let problem = problem_of(source, &index, self.truncation.get());
        let mut best = Best::of(&problem, seeds);
        let root = problem.measured(Cube {
            rotation: Vector::zeros(),
            translation: target.centroid().coords,
            half_rotation: std::f64::consts::PI,
            half_translation: self.reach.get(),
            lower: 0.0,
            value: f64::INFINITY,
        });
        best.offer(problem.pose(&root), root.value);
        Ok(self.search(&problem, target, root, best))
    }

    fn search(
        &self,
        problem: &Problem<'_>,
        target: &PointCloud,
        root: Cube,
        mut best: Best,
    ) -> Certificate {
        let mut open = BinaryHeap::from([root]);
        let mut settled: Vec<Cube> = Vec::new();
        let mut measured = 1;
        while measured < self.budget {
            let batch = self.next_batch(&mut open, &mut settled, problem.reach, best.value);
            if batch.is_empty() {
                break;
            }
            let children = map(&batch, |cube| problem.measured(*cube));
            measured += children.len();
            if let Some(lowest) = children
                .iter()
                .min_by(|left, right| left.value.total_cmp(&right.value))
            {
                best.offer(problem.pose(lowest), lowest.value);
                best.polish(problem, target);
            }
            open.extend(
                children
                    .into_iter()
                    .filter(|cube| cube.lower <= best.value + self.margin),
            );
        }
        let closed = open
            .iter()
            .all(|cube| cube.lower > best.value + self.margin);
        let floor = open
            .iter()
            .chain(&settled)
            .map(|cube| cube.lower)
            .fold(best.value + self.margin.min(0.0), f64::min);
        Certificate {
            lower: floor,
            upper: best.value,
            pose: best.pose,
            closed,
            measured,
            survivors: settled.iter().map(|cube| problem.pose(cube)).collect(),
        }
    }

    fn next_batch(
        &self,
        open: &mut BinaryHeap<Cube>,
        settled: &mut Vec<Cube>,
        reach: f64,
        best: f64,
    ) -> Vec<Cube> {
        let mut children = Vec::with_capacity(BATCH);
        while children.len() < BATCH {
            let Some(cube) = open.pop() else { break };
            if cube.lower > best + self.margin {
                open.clear();
                break;
            }
            match cube.split(reach, self.resolution) {
                Some(split) => children.extend(split),
                None => settled.push(cube),
            }
        }
        children
    }
}

fn problem_of<'data>(
    source: &PointCloud,
    index: &'data NearestNeighbors,
    truncation: f64,
) -> Problem<'data> {
    let centre = source.centroid().coords;
    let centred: Vec<Point> = source
        .points()
        .iter()
        .map(|point| Point::from(point.coords - centre))
        .collect();
    let reach = centred
        .iter()
        .map(|point| point.coords.norm())
        .fold(0.0, f64::max);
    Problem {
        centred,
        centre,
        reach,
        index,
        truncation_squared: truncation * truncation,
    }
}

/// The best pose evaluated so far: the upper bound.
struct Best {
    pose: RigidTransform,
    value: f64,
}

impl Best {
    fn of(problem: &Problem<'_>, seeds: &[RigidTransform]) -> Self {
        let mut best = Self {
            pose: RigidTransform::identity(),
            value: f64::INFINITY,
        };
        for seed in seeds {
            best.offer(*seed, problem.value(seed));
        }
        best
    }

    fn offer(&mut self, pose: RigidTransform, value: f64) {
        if value < self.value {
            self.pose = pose;
            self.value = value;
        }
    }

    /// A few closed-form steps on the pairs within the truncation: the bound tightens
    /// sooner from a good upper bound, and only an evaluated value is kept.
    fn polish(&mut self, problem: &Problem<'_>, target: &PointCloud) {
        let mut pose = self.pose;
        for _ in 0..POLISH_STEPS {
            let rotated = |point: &Point| pose.apply(&Point::from(point.coords + problem.centre));
            let (from, to): (Vec<Point>, Vec<Point>) = problem
                .centred
                .iter()
                .filter_map(|point| {
                    let found = problem.index.nearest(&rotated(point));
                    (found.squared_distance < problem.truncation_squared).then(|| {
                        (
                            Point::from(point.coords + problem.centre),
                            target.points()[found.index],
                        )
                    })
                })
                .unzip();
            let Ok(fit) = fit_rigid(&from, &to, None) else {
                return;
            };
            pose = fit.transform;
            self.offer(pose, problem.value(&pose));
        }
    }
}
