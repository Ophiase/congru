use congru_sim::Degradation;
use congru_synth::sequence::Handheld;

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
    pub handheld: Handheld,
    /// A can passing between the camera and the part.
    pub occluder: bool,
    /// Each of the two abrupt turns of the table, in degrees.
    pub knock_degrees: f64,
}

/// A camera in a child's hands: several degrees and centimetres of slow
/// wander, over about two seconds.
const HANDHELD: Handheld = Handheld {
    angle: 0.07,
    shift: 0.02,
    period: 45.0,
};

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
    handheld: Handheld {
        angle: 0.0,
        shift: 0.0,
        period: 0.0,
    },
    occluder: false,
    knock_degrees: 0.0,
};

/// Every effect demo A certifies, at once.
const ALL: Condition = Condition {
    name: "all effects",
    degradation: Degradation {
        lens: 0.08,
        ..SENSOR
    },
    flex: 0.004,
    shake: 1.0,
    handheld: HANDHELD,
    occluder: true,
    knock_degrees: 70.0,
    ..BASE
};

/// The first two are drawn together; each other one alone.
pub const CONDITIONS: [Condition; 7] = [
    ALL,
    Condition {
        name: "all in a room",
        clutter: Clutter::Room,
        ..ALL
    },
    BASE,
    Condition {
        name: "shake",
        shake: 1.0,
        handheld: HANDHELD,
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
];
