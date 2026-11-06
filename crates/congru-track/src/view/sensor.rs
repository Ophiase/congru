use congru::{Error, OrientedCloud, RigidTransform, Target, Vector};
use congru_vision::visible_from_origin;

use crate::ModelView;

/// A model seen by a sensor at the origin of the scan's frame: a followed
/// frame is aligned only onto the side of the model facing the sensor at
/// the predicted pose, so points of one face of a thin plate never pair
/// with the face behind it.
#[derive(Debug)]
pub struct SensorView {
    model: OrientedCloud,
    whole: Target,
}

impl SensorView {
    /// # Errors
    /// As [`Target::from_oriented`].
    pub fn new(model: OrientedCloud) -> Result<Self, Error> {
        Ok(Self {
            whole: Target::from_oriented(model.clone())?,
            model,
        })
    }
}

impl ModelView for SensorView {
    fn whole(&self) -> &Target {
        &self.whole
    }

    fn seen_from(&self, pose: &RigidTransform) -> Result<Option<Target>, Error> {
        let placed: Vec<_> = self
            .model
            .cloud()
            .points()
            .iter()
            .map(|point| pose.apply(point))
            .collect();
        let visible = visible_from_origin(&placed);
        let points: Vec<_> = visible
            .iter()
            .map(|index| self.model.cloud().points()[*index])
            .collect();
        let normals: Vec<Vector> = visible
            .iter()
            .map(|index| self.model.normals()[*index].into_inner())
            .collect();
        Target::from_oriented(OrientedCloud::new(
            congru::PointCloud::new(points)?,
            &normals,
        )?)
        .map(Some)
    }
}
