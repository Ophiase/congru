mod camera;
mod sensor;

pub use camera::CameraView;
pub use sensor::SensorView;

use congru::{Error, RigidTransform, Target};

/// What the tracker aligns a frame onto. Poses map the model's frame to the
/// sensor's.
pub trait ModelView: std::fmt::Debug {
    /// The whole model, for finding it without a pose.
    fn whole(&self) -> &Target;

    /// The model as the sensor would see it from `pose`, in the model's
    /// frame, when that differs from the whole: a camera returns only the
    /// visible faces, so hidden ones cannot attract points. `None` aligns
    /// onto the whole model.
    ///
    /// # Errors
    /// When the view cannot be built, such as a pose that shows nothing.
    fn seen_from(&self, pose: &RigidTransform) -> Result<Option<Target>, Error> {
        let _ = pose;
        Ok(None)
    }
}

/// A model aligned onto as a whole, whatever the pose.
#[derive(Debug)]
pub struct WholeModel(pub Target);

impl ModelView for WholeModel {
    fn whole(&self) -> &Target {
        &self.0
    }
}
