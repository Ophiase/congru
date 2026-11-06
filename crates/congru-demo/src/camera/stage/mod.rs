use congru::{Point, RigidTransform, TriangleMesh};
use std::f64::consts::{FRAC_PI_2, TAU};

use congru::Vector;
use congru_sim::{Degradation, Turntable, flexed};
use congru_vision::pixel::coordinate;
use congru_vision::{Camera, DepthImage, Frame, render};

mod crop;
mod follow;
pub(crate) mod hold;

use super::conditions::{Clutter, Condition};
use super::detect::{Detector, region_of_interest};
pub use follow::Answer;
use follow::Follower;

const PART_SPAN: f64 = 0.25;
const MODEL_SPACING: f64 = 0.003;
const SCAN_VOXEL: f64 = 0.004;
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
    pub answer: Answer,
}

impl Stage {
    /// This stage on a table that turns this much over the sequence.
    pub fn turning(&self, turns: f64) -> Self {
        Self {
            part: self.part.clone(),
            turntable: self.turntable.clone(),
            surroundings: self.surroundings.clone(),
            occluder: self.occluder.clone(),
            camera: self.camera,
            path: Turntable { turns, ..self.path },
        }
    }

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

    pub fn run(&self, condition: &Condition) -> anyhow::Result<Vec<Step>> {
        self.run_from(condition, |detector, depth, region| {
            detector.find_in(depth, region, &self.camera)
        })
    }

    /// Runs the tracker, asking `start` for a pose whenever it has none.
    pub fn run_from(
        &self,
        condition: &Condition,
        mut start: impl FnMut(&Detector, &DepthImage, &DepthImage) -> Option<RigidTransform>,
    ) -> anyhow::Result<Vec<Step>> {
        let path = self.path_for(condition);
        let mut follower = Follower::new(self)?;
        (0..path.frames)
            .map(|frame| {
                let truth = path.pose(frame);
                let part = flexed(&self.part, condition.flex, PART_SPAN, frame)?;
                let exact =
                    self.shoot(&part, &truth, &self.around(condition, &path, &truth, frame));
                let depth = condition.degradation.apply(&exact.depth, frame as u64);
                let region =
                    region_of_interest(&depth, &self.camera, &self.part, &truth, frame as u64);
                let from = follower
                    .forecast()
                    .or_else(|| start(&follower.detector, &depth, &region));
                let (scan, answer) = follower.step(self, (&depth, &region), from);
                Ok(Step {
                    image: camera_noise(condition).apply(&exact.depth, frame as u64 + 1),
                    depth,
                    scan: scan
                        .map(|cloud| cloud.points().to_vec())
                        .unwrap_or_default(),
                    truth,
                    answer,
                })
            })
            .collect()
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
