use serde::Deserialize;

use crate::props::Kind;

/// A scene as written in its TOML file: a camera, a ground, objects; one of
/// them is the target the methods must find. World frame: z up, metres.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Description {
    pub name: String,
    pub summary: String,
    pub camera: CameraSpec,
    pub ground: Option<GroundSpec>,
    /// Depth noise at one metre of a sensor whose noise grows as the square of the
    /// depth, in metres; none keeps the viewer's own.
    pub noise_at_one_metre: Option<f64>,
    pub objects: Vec<ObjectSpec>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraSpec {
    pub position: [f64; 3],
    pub look_at: [f64; 3],
    /// Horizontal, in degrees.
    pub field_of_view: f64,
    pub width: usize,
    pub height: usize,
}

/// A square of fractal relief under everything.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundSpec {
    pub size: f64,
    /// Largest height of the relief.
    pub relief: f64,
    /// Wavelength of its largest bumps.
    pub wave: f64,
    #[serde(default)]
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSpec {
    pub kind: Kind,
    #[serde(default = "unit_size")]
    pub size: [f64; 3],
    pub position: [f64; 3],
    /// Turn about the vertical, in degrees.
    #[serde(default)]
    pub yaw: f64,
    /// Tilt about the object's own x axis, in degrees, after the yaw.
    #[serde(default)]
    pub roll: f64,
    #[serde(default)]
    pub target: bool,
}

const fn unit_size() -> [f64; 3] {
    [1.0, 1.0, 1.0]
}
