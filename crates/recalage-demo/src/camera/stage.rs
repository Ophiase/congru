use recalage::{RigidTransform, TriangleMesh};
use recalage_sim::Turntable;
use recalage_vision::{Camera, DepthImage, Frame, back_project, render};

use super::conditions::Condition;
use super::tracking::{Estimate, Tracker};

/// The scene every condition is run on: the part, its turntable, the
/// camera, the path.
#[derive(Debug)]
pub struct Stage {
    pub part: TriangleMesh,
    pub turntable: TriangleMesh,
    pub camera: Camera,
    pub path: Turntable,
}

/// One frame of one run: what the sensor gave, the truth, and the tracker's
/// answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub depth: DepthImage,
    pub truth: RigidTransform,
    pub estimate: Option<Estimate>,
}

impl Stage {
    pub fn run(&self, condition: &Condition) -> anyhow::Result<Vec<Step>> {
        let mut tracker = Tracker::new(&self.part, self.camera)?;
        (0..self.path.frames)
            .map(|frame| {
                let truth = self.path.pose(frame);
                let exact = self.shoot(&truth, condition.clutter);
                let depth = condition.degradation.apply(&exact.depth, frame as u64);
                let estimate = tracker.update(&back_project(&depth, &self.camera)).ok();
                Ok(Step {
                    depth,
                    truth,
                    estimate,
                })
            })
            .collect()
    }

    pub fn shoot(&self, pose: &RigidTransform, clutter: bool) -> Frame {
        let mut scene = vec![(&self.part, *pose)];
        if clutter {
            scene.push((&self.turntable, *pose));
        }
        render(&self.camera, &scene)
    }
}
