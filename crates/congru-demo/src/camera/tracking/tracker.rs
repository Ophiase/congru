use congru::{
    Alignment, Convergence, Distance, Error, Icp, Kernel, Multistart, Point, PointCloud,
    RigidTransform, Target, TriangleMesh,
};
use congru_vision::{Camera, back_project, render, within, without_support};

use super::motion::{Extent, Motion};

const MODEL_SPACING: f64 = 0.003;
const VOXEL: f64 = 0.004;
const VISIBLE_VOXEL: f64 = 0.003;
const NORMAL_NEIGHBORS: usize = 10;
const SUPPORT_TOLERANCE: f64 = 0.004;
const KERNEL_SCALE: f64 = 0.008;
const CORRESPONDENCE_LIMIT: f64 = 0.02;
const FIRST_FRAME_STARTS: usize = 96;
const ITERATIONS_PER_FRAME: u32 = 12;
const LOST_INLIER_SHARE: f64 = 0.8;

/// Follows one part from frame to frame. The first frame has no pose and is
/// found by multistart against the whole model. Every later frame starts
/// from the last pose moved by the last velocity, and is aligned only onto
/// the part of the model visible from there: hidden faces cannot attract.
#[derive(Debug)]
pub struct Tracker {
    model: TriangleMesh,
    camera: Camera,
    whole: Target,
    extent: Extent,
    local: Icp,
    global: Multistart,
    motion: Motion,
}

/// One frame's answer: the part's pose in the camera frame, and how the
/// alignment that produced it went.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub pose: RigidTransform,
    pub alignment: Alignment,
}

impl Tracker {
    pub fn new(model: &TriangleMesh, camera: Camera) -> Result<Self, Error> {
        Ok(Self {
            model: model.clone(),
            camera,
            whole: Target::from_mesh(model, Distance::new(MODEL_SPACING)?)?,
            extent: Extent::of(model.vertices()),
            local: Icp {
                kernel: Kernel::Cauchy(Distance::new(KERNEL_SCALE)?),
                max_correspondence_distance: Some(Distance::new(CORRESPONDENCE_LIMIT)?),
                convergence: Convergence {
                    max_iterations: ITERATIONS_PER_FRAME,
                    ..Convergence::default()
                },
                ..Icp::default()
            },
            global: Multistart {
                starts: FIRST_FRAME_STARTS,
                icp: Icp {
                    kernel: Kernel::Cauchy(Distance::new(KERNEL_SCALE)?),
                    ..Icp::default()
                },
            },
            motion: Motion::default(),
        })
    }

    /// `seen` holds the points of one depth frame, in the camera frame.
    pub fn update(&mut self, seen: &[Point]) -> Result<Estimate, Error> {
        let predicted = self.motion.predict();
        let ground_free = without_support(seen, SUPPORT_TOLERANCE, 2.0 * self.extent.reach);
        let focus = match predicted {
            Some(pose) => pose.apply(&self.extent.centre),
            None => PointCloud::new(ground_free.clone())?.centroid(),
        };
        let scan = PointCloud::new(within(&ground_free, &focus, self.extent.reach))?
            .voxel_downsample(Distance::new(VOXEL)?);
        let attempt = match predicted {
            Some(pose) => self
                .visible(&pose)
                .and_then(|target| self.local.align(&scan, &target, &pose.inverse())),
            None => self.global.align(&scan, &self.whole),
        };
        let alignment = attempt.inspect_err(|_| self.motion.lose())?;
        let pose = alignment.transform.inverse();
        if alignment.inlier_fraction < LOST_INLIER_SHARE {
            self.motion.lose();
        } else {
            self.motion.record(pose);
        }
        Ok(Estimate { pose, alignment })
    }

    /// The model as the camera would see it at `pose`, in the part's frame.
    fn visible(&self, pose: &RigidTransform) -> Result<Target, Error> {
        let view = render(&self.camera, &[(&self.model, *pose)]);
        let seen = PointCloud::new(back_project(&view.depth, &self.camera))?
            .transformed(&pose.inverse())
            .voxel_downsample(Distance::new(VISIBLE_VOXEL)?);
        Target::with_estimated_normals(seen, NORMAL_NEIGHBORS)
    }
}
