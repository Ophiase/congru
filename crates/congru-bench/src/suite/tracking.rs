use congru_synth::sequence::{Drift, Jump, Sequence as Frames, Shake};

const FRAMES: usize = 80;

/// One sequence of the tracking suite, for a model of size `span`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: &'static str,
    pub frames: Frames,
}

/// Sparse, noisy and cluttered already; each sequence adds one trouble.
#[must_use]
pub fn sequences(span: f64) -> Vec<Sequence> {
    let base = baseline(span);
    let shake = Shake {
        angle: 0.0105,
        shift: 0.02 * span,
    };
    let added = |name, frames| Sequence { name, frames };
    let mut all = vec![
        added("noise", base.clone()),
        added(
            "shake",
            Frames {
                shake,
                ..base.clone()
            },
        ),
        added(
            "bend",
            Frames {
                bend: 0.02 * span,
                ..base.clone()
            },
        ),
    ];
    all.extend(hidden(&base, span));
    all
}

fn hidden(base: &Frames, span: f64) -> [Sequence; 3] {
    let added = |name, frames| Sequence { name, frames };
    [
        added(
            "occluder",
            Frames {
                occluder: 0.3 * span,
                ..base.clone()
            },
        ),
        added(
            "lens",
            Frames {
                distortion: 1.0,
                ..base.clone()
            },
        ),
        added(
            "knocks",
            Frames {
                jumps: knocks(span),
                ..base.clone()
            },
        ),
    ]
}

fn baseline(span: f64) -> Frames {
    let drift = Drift {
        distance: 2.5 * span,
        turns: 0.5,
        sway: 0.2 * span,
    };
    Frames {
        density: 0.1,
        noise: 0.025 * span,
        clutter: 40,
        ..Frames::still(FRAMES, drift)
    }
}

fn knocks(span: f64) -> Vec<Jump> {
    vec![
        Jump {
            frame: 30,
            turn: [0.0, 1.2, 0.3],
            shift: [0.15 * span, 0.0, 0.0],
        },
        Jump {
            frame: 60,
            turn: [0.8, -0.6, 0.0],
            shift: [-0.2 * span, 0.1 * span, 0.0],
        },
    ]
}
