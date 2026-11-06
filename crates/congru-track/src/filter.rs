use congru::{MotionKind, RigidTransform, WeakMotion};
use nalgebra::UnitQuaternion;
use nalgebra::{Vector3, Vector6};

/// How far each correction moves the state toward the measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gains {
    /// Share of the innovation taken into the pose: 1 trusts each frame.
    pub pose: f64,
    /// Share of the innovation taken into the velocity, per frame.
    pub velocity: f64,
}

/// How an object is expected to move, the forecast's one assumption: a part on a
/// table, a thing thrown and a car are not the same [`MotionModel`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionModel {
    pub rotation: Gains,
    pub translation: Gains,
    /// The share of the velocity left after a frame, as in the [Singer model]: 1
    /// goes on for ever, 0 stands where last seen, between dies away.
    ///
    /// [Singer model]: https://doi.org/10.1109/TAES.1970.310128
    pub persistence: f64,
}

impl MotionModel {
    /// The same gains on both, a constant velocity.
    #[must_use]
    pub const fn uniform(gains: Gains) -> Self {
        Self {
            rotation: gains,
            translation: gains,
            persistence: 1.0,
        }
    }

    /// As it is, for something standing still whatever the frames say.
    #[must_use]
    pub const fn at_rest(self) -> Self {
        Self {
            persistence: 0.0,
            ..self
        }
    }

    fn poses(&self) -> Vector6<f64> {
        let (turn, shift) = (self.rotation.pose, self.translation.pose);
        Vector6::new(turn, turn, turn, shift, shift, shift)
    }

    fn velocities(&self) -> Vector6<f64> {
        let (turn, shift) = (self.rotation.velocity, self.translation.velocity);
        Vector6::new(turn, turn, turn, shift, shift, shift)
    }
}

/// An alpha-beta filter on rigid motions, the steady-state Kalman filter of
/// a constant-velocity model. Increments are written as six-vectors,
/// rotation vector then translation, in the model's own frame, so a turn of
/// the model is a turn about the model and not about the sensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionFilter {
    model: MotionModel,
    pose: Option<RigidTransform>,
    velocity: Vector6<f64>,
}

impl MotionFilter {
    #[must_use]
    pub fn new(gains: Gains) -> Self {
        Self::of(MotionModel::uniform(gains))
    }

    #[must_use]
    pub fn of(model: MotionModel) -> Self {
        Self {
            model,
            pose: None,
            velocity: Vector6::zeros(),
        }
    }

    /// The pose expected at the next frame, if there is one to start from.
    #[must_use]
    pub fn predict(&self) -> Option<RigidTransform> {
        self.pose
            .map(|pose| exponential(&self.velocity).then(&pose))
    }

    /// Takes in `measured`, except along the motions the frame left free,
    /// where the prediction is kept; returns the corrected pose.
    pub fn correct(&mut self, measured: &RigidTransform, free: &[WeakMotion]) -> RigidTransform {
        let Some(predicted) = self.predict() else {
            self.pose = Some(*measured);
            return *measured;
        };
        let innovation = without(&logarithm(&measured.then(&predicted.inverse())), free);
        let corrected =
            exponential(&innovation.component_mul(&self.model.poses())).then(&predicted);
        self.velocity = self.velocity * self.model.persistence
            + innovation.component_mul(&self.model.velocities());
        self.pose = Some(corrected);
        corrected
    }

    pub fn reset(&mut self) {
        self.pose = None;
        self.velocity = Vector6::zeros();
    }
}

fn logarithm(motion: &RigidTransform) -> Vector6<f64> {
    let rotation = motion.rotation().scaled_axis();
    let translation = motion.translation();
    Vector6::new(
        rotation.x,
        rotation.y,
        rotation.z,
        translation.x,
        translation.y,
        translation.z,
    )
}

fn exponential(twist: &Vector6<f64>) -> RigidTransform {
    RigidTransform::from_parts(
        UnitQuaternion::from_scaled_axis(twist.fixed_rows::<3>(0).into_owned()),
        twist.fixed_rows::<3>(3).into_owned(),
    )
}

/// Takes out of `innovation` every free motion: a slide along its
/// direction, or a turn about its axis with the shift that turning about a
/// point other than the origin implies.
fn without(innovation: &Vector6<f64>, free: &[WeakMotion]) -> Vector6<f64> {
    let mut rotation: Vector3<f64> = innovation.fixed_rows::<3>(0).into_owned();
    let mut translation: Vector3<f64> = innovation.fixed_rows::<3>(3).into_owned();
    for motion in free {
        match motion.kind {
            MotionKind::Rotation { axis, through } => {
                let amount = axis.dot(&rotation);
                rotation -= axis.into_inner() * amount;
                translation -= through.coords.cross(&axis) * amount;
            }
            MotionKind::Translation { direction } => {
                translation -= direction.into_inner() * direction.dot(&translation);
            }
        }
    }
    Vector6::new(
        rotation.x,
        rotation.y,
        rotation.z,
        translation.x,
        translation.y,
        translation.z,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAINS: Gains = Gains {
        pose: 0.6,
        velocity: 0.2,
    };

    fn moved(step: f64) -> RigidTransform {
        RigidTransform::from_axis_angle(Vector3::zeros(), [step, 0.0, 0.0])
    }

    fn shift_after(mut filter: MotionFilter, frames: u32) -> f64 {
        for frame in 0..frames {
            filter.correct(&moved(f64::from(frame)), &[]);
        }
        filter
            .predict()
            .map_or(f64::NAN, |pose| pose.translation().x)
    }

    #[test]
    fn a_steady_motion_is_forecast_without_lag() {
        let forecast = shift_after(MotionFilter::new(GAINS), 60);
        assert!((forecast - 60.0).abs() < 1e-3, "{forecast}");
    }

    #[test]
    fn what_stands_still_is_forecast_near_its_last_place() {
        let model = MotionModel::uniform(GAINS).at_rest();
        let after = shift_after(MotionFilter::of(model), 60);
        let free = shift_after(MotionFilter::new(GAINS), 60);
        assert!(after < free - 1.0, "{after} against {free}");
    }

    #[test]
    fn translation_answers_faster_than_rotation_when_told_to() {
        let model = MotionModel {
            translation: Gains {
                pose: 1.0,
                velocity: 0.0,
            },
            rotation: Gains {
                pose: 0.0,
                velocity: 0.0,
            },
            persistence: 1.0,
        };
        let mut filter = MotionFilter::of(model);
        filter.correct(&RigidTransform::identity(), &[]);
        let measured = RigidTransform::from_axis_angle(Vector3::x() * 0.3, [0.5, 0.0, 0.0]);
        let corrected = filter.correct(&measured, &[]);
        assert!((corrected.translation().x - 0.5).abs() < 1e-9);
        assert!(corrected.rotation().angle() < 1e-9);
    }
}
