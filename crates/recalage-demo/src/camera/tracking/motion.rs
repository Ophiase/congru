use recalage::{Point, PointCloud, RigidTransform};

const CROP_MARGIN: f64 = 0.015;
const MAX_TURN_PER_FRAME: f64 = 0.15;
const MAX_SHIFT_PER_FRAME: f64 = 0.02;

/// Where the part sits in its own frame: the centre of its bounding box,
/// and how far from it a point of the part, or of its scan, can lie.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent {
    pub centre: Point,
    pub reach: f64,
}

impl Extent {
    pub fn of(vertices: &PointCloud) -> Self {
        let points = vertices.points();
        let (lowest, highest) = points.iter().fold(
            (points[0].coords, points[0].coords),
            |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
        );
        let centre = Point::from((lowest + highest) / 2.0);
        let reach = points
            .iter()
            .map(|point| (point - centre).norm())
            .fold(0.0, f64::max)
            + CROP_MARGIN;
        Self { centre, reach }
    }
}

/// The last pose and the motion between the last two, for a constant
/// velocity prediction. An implausible jump resets the velocity.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Motion {
    last: Option<RigidTransform>,
    velocity: Option<RigidTransform>,
}

impl Motion {
    pub fn predict(&self) -> Option<RigidTransform> {
        self.last
            .map(|last| self.velocity.map_or(last, |velocity| last.then(&velocity)))
    }

    pub fn record(&mut self, pose: RigidTransform) {
        self.velocity = self
            .last
            .map(|last| last.inverse().then(&pose))
            .filter(|step| {
                step.rotation().angle() < MAX_TURN_PER_FRAME
                    && step.translation().norm() < MAX_SHIFT_PER_FRAME
            });
        self.last = Some(pose);
    }

    pub fn lose(&mut self) {
        *self = Self::default();
    }
}
