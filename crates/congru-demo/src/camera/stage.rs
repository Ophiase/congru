use congru::{Distance, Point, PointCloud, RigidTransform, TriangleMesh};
use std::f64::consts::{FRAC_PI_2, TAU};

use congru::Vector;
use congru_sim::{Degradation, Turntable, flexed};
use congru_track::{Settings, Tracker, Update};
use congru_vision::pixel::coordinate;
use congru_vision::{
    Camera, DepthImage, Frame, around_view, back_project, in_view, render, within,
};

use super::conditions::{Clutter, Condition};
use super::detect::Detector;
use super::view::CameraView;

const PART_SPAN: f64 = 0.25;
const MODEL_SPACING: f64 = 0.003;
const SCAN_VOXEL: f64 = 0.004;
/// How far outside the predicted silhouette a return may lie, in pixels, and
/// how far from the predicted depth, in metres.
const VIEW_MARGIN: usize = 0;
const VIEW_GATE: f64 = 0.03;
const CROP_MARGIN: f64 = 0.015;
/// Returns this many pixels off the predicted silhouette stand around the
/// part, whatever a wrong prediction; below this share of the ball, the ball
/// holds little else than the part.
const SURROUNDINGS_MARGIN: usize = 12;
const MOST_AROUND: f64 = 0.02;
const OCCLUDER_PERIOD: f64 = 45.0;

/// The scene every condition is run on.
#[derive(Debug)]
pub struct Stage {
    pub part: TriangleMesh,
    pub turntable: TriangleMesh,
    pub surroundings: TriangleMesh,
    pub occluder: TriangleMesh,
    pub camera: Camera,
    pub path: Turntable,
}

/// One frame of one run: what the sensor gave, the truth, and the tracker's
/// answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub image: DepthImage,
    pub depth: DepthImage,
    pub scan: Vec<Point>,
    pub truth: RigidTransform,
    pub update: Option<Update>,
}

impl Stage {
    /// What the sensor returns at `frame`, and the truth to score it by.
    pub fn sensed(
        &self,
        condition: &Condition,
        frame: usize,
    ) -> anyhow::Result<(DepthImage, RigidTransform)> {
        let path = self.path_for(condition);
        let truth = path.pose(frame);
        let part = flexed(&self.part, condition.flex, PART_SPAN, frame)?;
        let exact = self.shoot(&part, &truth, &self.around(condition, &path, &truth, frame));
        Ok((
            condition.degradation.apply(&exact.depth, frame as u64),
            truth,
        ))
    }

    /// The share of a ball around `pose` that stands away from the part.
    fn share_around(&self, depth: &DepthImage, pose: &RigidTransform) -> f64 {
        let (centre, reach) = extent(&self.part);
        let focus = pose.apply(&centre);
        let rendered = render(&self.camera, &[(&self.part, *pose)]).depth;
        let around = around_view(depth, &rendered, &self.camera, SURROUNDINGS_MARGIN);
        let ball = within(&back_project(depth, &self.camera), &focus, reach).len();
        coordinate(within(&around, &focus, reach).len()) / coordinate(ball.max(1))
    }

    /// As [`Stage::share_around`], at the truth, for measuring.
    pub fn share_around_truth(&self, condition: &Condition, frame: usize) -> anyhow::Result<f64> {
        let (depth, truth) = self.sensed(condition, frame)?;
        Ok(self.share_around(&depth, &truth))
    }

    pub fn run(&self, condition: &Condition) -> anyhow::Result<Vec<Step>> {
        self.run_from(condition, |detector, depth| {
            detector.find(depth, &self.camera)
        })
    }

    /// Runs the tracker, asking `start` for a pose whenever it has none.
    pub fn run_from(
        &self,
        condition: &Condition,
        mut start: impl FnMut(&Detector, &DepthImage) -> Option<RigidTransform>,
    ) -> anyhow::Result<Vec<Step>> {
        let view = CameraView::new(&self.part, self.camera, Distance::new(MODEL_SPACING)?)?;
        let mut tracker = Tracker::new("tracker", view, Settings::for_part(PART_SPAN)?);
        let path = self.path_for(condition);
        let detector = Detector::new(&self.part, PART_SPAN)?;
        let mut last: Option<RigidTransform> = None;
        (0..path.frames)
            .map(|frame| {
                let truth = path.pose(frame);
                let part = flexed(&self.part, condition.flex, PART_SPAN, frame)?;
                let exact =
                    self.shoot(&part, &truth, &self.around(condition, &path, &truth, frame));
                let depth = condition.degradation.apply(&exact.depth, frame as u64);
                let scan = last
                    .or_else(|| start(&detector, &depth))
                    .and_then(|start| self.scan_for(&detector, &depth, &start));
                let update = scan
                    .as_ref()
                    .and_then(|(start, cloud)| tracker.update_from(cloud, Some(*start)).ok());
                last = update
                    .as_ref()
                    .filter(|update| update.trusted)
                    .map(|update| update.pose);
                Ok(Step {
                    image: camera_noise(condition).apply(&exact.depth, frame as u64 + 1),
                    depth,
                    scan: scan
                        .map(|(_, cloud)| cloud.points().to_vec())
                        .unwrap_or_default(),
                    truth,
                    update,
                })
            })
            .collect()
    }

    /// The scan to align and the pose it starts from. A ball around the part
    /// keeps the edges that correct a slide along a flat face, so it stays while
    /// it holds little else than the part. Otherwise the crop keeps to the
    /// silhouette, and the pose is first brought along rays, which what stands
    /// around cannot pull.
    fn scan_for(
        &self,
        detector: &Detector,
        depth: &DepthImage,
        predicted: &RigidTransform,
    ) -> Option<(RigidTransform, PointCloud)> {
        let (centre, reach) = extent(&self.part);
        let points = back_project(depth, &self.camera);
        let ball = within(&points, &predicted.apply(&centre), reach);
        if self.share_around(depth, predicted) <= MOST_AROUND {
            let cloud = PointCloud::new(ball).ok()?;
            let thinned = cloud.voxel_downsample(Distance::new(SCAN_VOXEL).ok()?);
            return Some((*predicted, thinned));
        }
        let start = detector.along_rays(depth, &self.camera, predicted);
        Some((start, self.scan_in_view(depth, &start)?))
    }

    /// The returns the part, as predicted, can explain.
    fn scan_in_view(&self, depth: &DepthImage, predicted: &RigidTransform) -> Option<PointCloud> {
        let rendered = render(&self.camera, &[(&self.part, *predicted)]).depth;
        Some(
            PointCloud::new(in_view(
                depth,
                &rendered,
                &self.camera,
                VIEW_MARGIN,
                VIEW_GATE,
            ))
            .ok()?
            .voxel_downsample(Distance::new(SCAN_VOXEL).ok()?),
        )
    }

    fn path_for(&self, condition: &Condition) -> Turntable {
        Turntable {
            shake_degrees: condition.shake,
            handheld: condition.handheld,
            knock_degrees: condition.knock_degrees,
            ..self.path
        }
    }

    pub fn shoot(
        &self,
        part: &TriangleMesh,
        pose: &RigidTransform,
        others: &[(&TriangleMesh, RigidTransform)],
    ) -> Frame {
        let mut scene = vec![(part, *pose)];
        scene.extend_from_slice(others);
        render(&self.camera, &scene)
    }

    /// What else is in view at `frame`: the room, still in the world, and
    /// the can, crossing in front twice.
    pub(super) fn around(
        &self,
        condition: &Condition,
        path: &Turntable,
        pose: &RigidTransform,
        frame: usize,
    ) -> Vec<(&TriangleMesh, RigidTransform)> {
        let room = path.room(frame);
        let mut others = Vec::new();
        if condition.clutter == Clutter::Room {
            others.push((&self.turntable, *pose));
            others.push((&self.surroundings, room));
        }
        if condition.occluder {
            let phase = coordinate(frame) / OCCLUDER_PERIOD;
            let crossing = RigidTransform::from_axis_angle(
                Vector::x() * FRAC_PI_2,
                [0.2 * (TAU * phase).sin(), 0.04, 0.3],
            );
            others.push((&self.occluder, crossing.then(&room)));
        }
        others
    }
}

/// The colour camera: sharper than the depth, a little grain, the same lens.
fn camera_noise(condition: &Condition) -> Degradation {
    Degradation {
        blur: 0,
        noise: 0.002,
        holes: 0.0,
        flying_pixels: false,
        lens: condition.degradation.lens,
    }
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
