use std::path::Path;

use congru::{RigidTransform, TriangleMesh, Vector};
use congru_vision::{Camera, Frame, render};
use nalgebra::{Matrix3, Rotation3, UnitQuaternion};

use crate::SceneError;
use crate::description::{CameraSpec, Description, ObjectSpec};
use crate::noise::Fractal;
use crate::props::{Kind, ground};

const GROUND_SAMPLES: u32 = 120;
const GROUND_OCTAVES: u32 = 5;

/// A built scene: meshes placed in the world, and the camera looking at
/// them.
#[derive(Debug)]
pub struct Scene {
    pub name: String,
    pub summary: String,
    pub camera: Camera,
    /// Maps the world to the camera's frame (x right, y down, z forward).
    pub view: RigidTransform,
    pub objects: Vec<Placed>,
    /// Depth noise at one metre, when the scene states its sensor.
    pub noise_at_one_metre: Option<f64>,
    target: usize,
}

/// One object of a scene, in its own frame, and where it stands.
#[derive(Debug)]
pub struct Placed {
    pub kind: Option<Kind>,
    pub mesh: TriangleMesh,
    /// Maps the object's frame to the world.
    pub pose: RigidTransform,
}

impl Scene {
    /// # Errors
    /// When the file cannot be read or parsed, or has no single target.
    pub fn load(path: &Path) -> Result<Self, SceneError> {
        let name = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|source| SceneError::Read {
            path: name.clone(),
            source,
        })?;
        let description: Description =
            toml::from_str(&text).map_err(|source| SceneError::Parse { path: name, source })?;
        Self::build(&description)
    }

    /// # Errors
    /// When the description has no single target.
    pub fn build(description: &Description) -> Result<Self, SceneError> {
        let targets: Vec<usize> = (0..description.objects.len())
            .filter(|index| description.objects[*index].target)
            .collect();
        let [target] = targets[..] else {
            return Err(SceneError::Target {
                found: targets.len(),
            });
        };
        let mut objects = description
            .objects
            .iter()
            .map(place)
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(spec) = &description.ground {
            let relief = Fractal {
                octaves: GROUND_OCTAVES,
                wave: spec.wave,
                amplitude: spec.relief,
                seed: spec.seed,
            };
            objects.push(Placed {
                kind: None,
                mesh: ground(spec.size, GROUND_SAMPLES, &relief)?,
                pose: RigidTransform::identity(),
            });
        }
        Ok(Self {
            name: description.name.clone(),
            summary: description.summary.clone(),
            noise_at_one_metre: description.noise_at_one_metre,
            camera: Camera::with_field_of_view(
                description.camera.width,
                description.camera.height,
                description.camera.field_of_view,
            ),
            view: looking(&description.camera),
            objects,
            target,
        })
    }

    #[must_use]
    pub fn target(&self) -> &Placed {
        &self.objects[self.target]
    }

    /// Maps the target's own frame to the camera's: the pose a method must
    /// find.
    #[must_use]
    pub fn truth(&self) -> RigidTransform {
        self.target().pose.then(&self.view)
    }

    /// What the camera sees: every object, the ground included.
    #[must_use]
    pub fn render(&self) -> Frame {
        let placed: Vec<(&TriangleMesh, RigidTransform)> = self
            .objects
            .iter()
            .map(|object| (&object.mesh, object.pose.then(&self.view)))
            .collect();
        render(&self.camera, &placed)
    }

    /// The target alone, as the camera would see it with nothing in front.
    #[must_use]
    pub fn render_target(&self) -> Frame {
        render(&self.camera, &[(&self.target().mesh, self.truth())])
    }
}

fn place(spec: &ObjectSpec) -> Result<Placed, SceneError> {
    let turn = UnitQuaternion::from_axis_angle(&Vector::z_axis(), spec.yaw.to_radians())
        * UnitQuaternion::from_axis_angle(&Vector::x_axis(), spec.roll.to_radians());
    Ok(Placed {
        kind: Some(spec.kind),
        mesh: spec.kind.mesh(spec.size)?,
        pose: RigidTransform::from_parts(turn, spec.position),
    })
}

/// The world-to-camera motion of a camera at `position` looking at
/// `look_at`, the world's z up.
fn looking(spec: &CameraSpec) -> RigidTransform {
    let position = Vector::from(spec.position);
    let forward = (Vector::from(spec.look_at) - position).normalize();
    let right = forward
        .cross(&Vector::z())
        .try_normalize(f64::EPSILON)
        .unwrap_or_else(Vector::x);
    let down = forward.cross(&right);
    let rows = Matrix3::from_rows(&[right.transpose(), down.transpose(), forward.transpose()]);
    let rotation = UnitQuaternion::from_rotation_matrix(&Rotation3::from_matrix_unchecked(rows));
    RigidTransform::from_parts(rotation, -(rotation * position))
}
