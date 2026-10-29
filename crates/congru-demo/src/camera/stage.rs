use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use congru_sim::{Turntable, flexed};
use congru_track::{Settings, Tracker, Update};
use congru_vision::{Camera, DepthImage, Frame, back_project, render, within, without_support};

use super::conditions::{Clutter, Condition};
use super::view::CameraView;

const PART_SPAN: f64 = 0.25;
const SCAN_VOXEL: f64 = 0.004;
const SUPPORT_TOLERANCE: f64 = 0.004;
const CROP_MARGIN: f64 = 0.015;
/// How far from the truth a detector places the part on the first frame.
const FIRST_FOCUS_ERROR: [f64; 3] = [0.02, -0.015, 0.01];

/// The scene every condition is run on.
#[derive(Debug)]
pub struct Stage {
    pub part: TriangleMesh,
    pub turntable: TriangleMesh,
    pub surroundings: TriangleMesh,
    pub camera: Camera,
    pub path: Turntable,
}

/// One frame of one run: what the sensor gave, the truth, and the tracker's
/// answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub depth: DepthImage,
    pub truth: RigidTransform,
    pub update: Option<Update>,
}

impl Stage {
    pub fn run(&self, condition: &Condition) -> anyhow::Result<Vec<Step>> {
        let view = CameraView::new(&self.part, self.camera)?;
        let mut tracker = Tracker::new("tracker", view, Settings::for_part(PART_SPAN)?);
        let (centre, reach) = extent(&self.part);
        let path = Turntable {
            shake_degrees: condition.shake,
            handheld: condition.handheld,
            ..self.path
        };
        let mut last: Option<RigidTransform> = None;
        (0..path.frames)
            .map(|frame| {
                let truth = path.pose(frame);
                let part = flexed(&self.part, condition.flex, PART_SPAN, frame)?;
                let exact = self.shoot(&part, &truth, condition.clutter);
                let depth = condition.degradation.apply(&exact.depth, frame as u64);
                let focus = last.unwrap_or_else(|| truth.then(&nudge())).apply(&centre);
                let seen = without_support(
                    &back_project(&depth, &self.camera),
                    SUPPORT_TOLERANCE,
                    2.0 * reach,
                );
                let scan = PointCloud::new(within(&seen, &focus, reach))?
                    .voxel_downsample(Distance::new(SCAN_VOXEL)?);
                let update = tracker.update(&scan).ok();
                last = update
                    .as_ref()
                    .filter(|update| update.trusted)
                    .map(|update| update.pose)
                    .or(last);
                Ok(Step {
                    depth,
                    truth,
                    update,
                })
            })
            .collect()
    }

    pub fn shoot(&self, part: &TriangleMesh, pose: &RigidTransform, clutter: Clutter) -> Frame {
        let mut scene = vec![(part, *pose)];
        if clutter == Clutter::Room {
            scene.push((&self.turntable, *pose));
            scene.push((&self.surroundings, RigidTransform::identity()));
        }
        render(&self.camera, &scene)
    }
}

fn nudge() -> RigidTransform {
    RigidTransform::from_parts(nalgebra::UnitQuaternion::identity(), FIRST_FOCUS_ERROR)
}

/// The centre of the part's bounding box, and how far from it its points
/// and their scan can lie.
fn extent(part: &TriangleMesh) -> (Point, f64) {
    let points = part.vertices().points();
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
    (centre, reach)
}
