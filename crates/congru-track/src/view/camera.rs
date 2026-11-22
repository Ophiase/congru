use congru::{Distance, Error, PointCloud, RigidTransform, Target, TriangleMesh};
use congru_vision::{Camera, back_project, render};

use super::ModelView;

const NORMAL_NEIGHBORS: usize = 10;

/// The part as a depth camera sees it: a followed frame is aligned onto
/// the faces the camera would see at the predicted pose, rendered, so the
/// back of a plate or the underside of the base cannot attract points.
#[derive(Debug)]
pub struct CameraView {
    model: TriangleMesh,
    camera: Camera,
    whole: Target,
    spacing: Distance,
}

impl CameraView {
    /// `spacing` sets the sampling of the model and of its visible side: a
    /// hundredth of its size is plenty.
    ///
    /// # Errors
    /// When the model cannot be sampled at `spacing`.
    pub fn new(model: &TriangleMesh, camera: Camera, spacing: Distance) -> Result<Self, Error> {
        Ok(Self {
            model: model.clone(),
            camera,
            whole: Target::from_mesh(model, spacing)?,
            spacing,
        })
    }
}

impl ModelView for CameraView {
    fn whole(&self) -> &Target {
        &self.whole
    }

    fn seen_from(&self, pose: &RigidTransform) -> Result<Option<Target>, Error> {
        let view = render(&self.camera, &[(&self.model, *pose)]);
        let seen = PointCloud::new(back_project(&view.depth, &self.camera))?
            .transformed(&pose.inverse())
            .voxel_downsample(self.spacing);
        Target::with_estimated_normals(seen, NORMAL_NEIGHBORS).map(Some)
    }
}
