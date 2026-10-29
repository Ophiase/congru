use congru_synth::sequence::{Drift, Handheld, Jump, Sequence, Shake};

pub const FRAMES: usize = 120;

const DRIFT: Drift = Drift {
    distance: 0.5,
    turns: 0.5,
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

/// Sparse, noisy and cluttered already; each scenario adds one more
/// trouble on top.
fn baseline() -> Sequence {
    Sequence {
        density: 0.1,
        noise: 0.005,
        clutter: 40,
        ..Sequence::still(FRAMES, DRIFT)
    }
}

const SHAKE: Shake = Shake {
    angle: 0.010_5,
    shift: 0.004,
};

/// A camera in a child's hands: several degrees and centimetres of slow
/// wander, over about two seconds.
const HANDHELD: Handheld = Handheld {
    angle: 0.1,
    shift: 0.025,
    period: 50.0,
};

pub fn scenarios() -> [Scenario; 6] {
    let base = baseline();
    let with = |name, sequence| Scenario { name, sequence };
    [
        with("noise", base.clone()),
        with("shake", shaken(&base)),
        with(
            "bend",
            Sequence {
                bend: 0.004,
                ..base.clone()
            },
        ),
        with(
            "occluder",
            Sequence {
                occluder: 0.06,
                ..base.clone()
            },
        ),
        with(
            "lens",
            Sequence {
                distortion: 1.0,
                ..base.clone()
            },
        ),
        with(
            "knocks",
            Sequence {
                jumps: KNOCKS.to_vec(),
                ..base
            },
        ),
    ]
}

/// The sensor jolting every frame, and wandering as in a hand.
fn shaken(base: &Sequence) -> Sequence {
    Sequence {
        shake: SHAKE,
        handheld: HANDHELD,
        ..base.clone()
    }
}
