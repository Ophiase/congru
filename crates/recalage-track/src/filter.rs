use nalgebra::UnitQuaternion;
use nalgebra::{Vector3, Vector6};
use recalage::{MotionKind, RigidTransform, WeakMotion};

/// How far each correction moves the state toward the measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gains {
    /// Share of the innovation taken into the pose: 1 trusts each frame.
    pub pose: f64,
    /// Share of the innovation taken into the velocity, per frame.
    pub velocity: f64,
}

/// An alpha-beta filter on rigid motions, the steady-state Kalman filter of
/// a constant-velocity model. Increments are written as six-vectors,
/// rotation vector then translation, in the model's own frame, so a turn of
/// the model is a turn about the model and not about the sensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionFilter {
    gains: Gains,
    pose: Option<RigidTransform>,
    velocity: Vector6<f64>,
}

impl MotionFilter {
    #[must_use]
    pub fn new(gains: Gains) -> Self {
        Self {
            gains,
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
        let corrected = exponential(&(innovation * self.gains.pose)).then(&predicted);
        self.velocity += innovation * self.gains.velocity;
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
