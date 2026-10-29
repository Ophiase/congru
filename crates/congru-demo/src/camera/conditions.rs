use congru_sim::Degradation;

/// What else the camera sees besides the part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clutter {
    /// The part alone, on black.
    Nothing,
    /// The turntable under the part, a wall behind, a bench, a box, a can.
    Room,
}

/// One way of seeing the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Condition {
    pub name: &'static str,
    pub degradation: Degradation,
    pub clutter: Clutter,
    /// Largest flexing of the part, in metres.
    pub flex: f64,
    /// Largest jolt of the camera per frame, in degrees.
    pub shake: f64,
}

/// A consumer depth camera at half a metre: noise, a little blur, a few
/// holes, flying pixels at edges. Every condition starts from it.
const SENSOR: Degradation = Degradation {
    blur: 1,
    noise: 0.01,
    holes: 0.04,
    flying_pixels: true,
    lens: 0.0,
};

const BASE: Condition = Condition {
    name: "sensor",
    degradation: SENSOR,
    clutter: Clutter::Nothing,
    flex: 0.0,
    shake: 0.0,
};

pub const CONDITIONS: [Condition; 6] = [
    BASE,
    Condition {
        name: "shake",
        shake: 1.0,
        ..BASE
    },
    Condition {
        name: "lens",
        degradation: Degradation {
            lens: 0.08,
            ..SENSOR
        },
        ..BASE
    },
    Condition {
        name: "flex",
        flex: 0.004,
        ..BASE
    },
    Condition {
        name: "room",
        clutter: Clutter::Room,
        ..BASE
    },
    Condition {
        name: "everything",
        degradation: Degradation {
            noise: 0.02,
            holes: 0.12,
            lens: 0.08,
            ..SENSOR
        },
        clutter: Clutter::Room,
        flex: 0.004,
        shake: 1.0,
    },
];
