use congru::{Distance, Error, PointCloud, RigidTransform, Target, TriangleMesh};
use congru_track::ModelView;
use congru_vision::{Camera, back_project, render};

const MODEL_SPACING: f64 = 0.003;
const VISIBLE_VOXEL: f64 = 0.003;
const NORMAL_NEIGHBORS: usize = 10;

/// The part as a depth camera sees it: a followed frame is aligned onto
/// the faces the camera would see at the predicted pose, rendered, so the
/// back of a plate or the underside of the base cannot attract points.
#[derive(Debug)]
pub struct CameraView {
    model: TriangleMesh,
    camera: Camera,
    whole: Target,
}

impl CameraView {
    pub fn new(model: &TriangleMesh, camera: Camera) -> Result<Self, Error> {
        Ok(Self {
            model: model.clone(),
            camera,
            whole: Target::from_mesh(model, Distance::new(MODEL_SPACING)?)?,
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
            .voxel_downsample(Distance::new(VISIBLE_VOXEL)?);
        Target::with_estimated_normals(seen, NORMAL_NEIGHBORS).map(Some)
    }
}
