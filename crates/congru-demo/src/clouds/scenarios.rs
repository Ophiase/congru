use congru_synth::sequence::{Drift, Jump, Sequence};

pub const FRAMES: usize = 120;

const DRIFT: Drift = Drift {
    distance: 0.5,
    turns: 0.6,
    sway: 0.04,
};

const KNOCKS: [Jump; 2] = [
    Jump {
        frame: 45,
        turn: [0.0, 1.2, 0.3],
        shift: [0.03, 0.0, 0.0],
    },
    Jump {
        frame: 90,
        turn: [0.8, -0.6, 0.0],
        shift: [-0.04, 0.02, 0.0],
    },
];

/// One generated test sequence, and its name on the animation.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub name: &'static str,
    pub sequence: Sequence,
}

pub fn scenarios() -> [Scenario; 4] {
    let calm = Sequence {
        frames: FRAMES,
        drift: DRIFT,
        jumps: Vec::new(),
        noise: 0.002,
        occluder: 0.0,
        distortion: 0.0,
    };
    [
        Scenario {
            name: "noise",
            sequence: Sequence {
                noise: 0.004,
                ..calm.clone()
            },
        },
        Scenario {
            name: "occluder",
            sequence: Sequence {
                occluder: 0.06,
                ..calm.clone()
            },
        },
        Scenario {
            name: "lens",
            sequence: Sequence {
                distortion: 1.0,
                ..calm.clone()
            },
        },
        Scenario {
            name: "knocks",
            sequence: Sequence {
                jumps: KNOCKS.to_vec(),
                ..calm
            },
        },
    ]
}
