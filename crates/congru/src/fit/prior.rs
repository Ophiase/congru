use nalgebra::{Matrix6, Vector6};

use crate::geometry::count_as_float;
use crate::{Error, Point, RigidTransform};

/// How far the pose may plausibly stray from where the alignment started:
/// the point-to-plane cost then adds the squared distance from the initial
/// pose, as the measurement update of a Kalman filter adds the prediction.
/// A pair whose residual is the sensor noise weighs as much as a stray of
/// one `rotation` or one `translation`, divided among all the pairs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prior {
    rotation: f64,
    translation: f64,
    noise: f64,
}

impl Prior {
    /// `rotation` in radians; `translation` and `noise` in the unit of the
    /// points.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] unless all three are finite and positive.
    pub fn new(rotation: f64, translation: f64, noise: f64) -> Result<Self, Error> {
        for (name, value) in [
            ("rotation", rotation),
            ("translation", translation),
            ("noise", noise),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(Error::InvalidParameter {
                    name,
                    expected: "finite and positive",
                    value,
                });
            }
        }
        Ok(Self {
            rotation,
            translation,
            noise,
        })
    }

    /// The terms added to the mean normal equations of `count` pairs, for a
    /// pose `deviation` away from the initial one, written about `centre`.
    pub(crate) fn pull(
        &self,
        count: usize,
        deviation: &RigidTransform,
        centre: &Point,
    ) -> (Matrix6<f64>, Vector6<f64>) {
        let share = self.noise * self.noise / count_as_float(count.max(1));
        let turn = share / (self.rotation * self.rotation);
        let shift = share / (self.translation * self.translation);
        let stiffness =
            Matrix6::from_diagonal(&Vector6::new(turn, turn, turn, shift, shift, shift));
        let rotation = deviation.rotation().scaled_axis();
        let translation = deviation.apply(centre) - centre;
        let away = Vector6::new(
            rotation.x,
            rotation.y,
            rotation.z,
            translation.x,
            translation.y,
            translation.z,
        );
        (stiffness, stiffness * away)
    }
}
