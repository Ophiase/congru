#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

use congru::{Distance, OrientedCloud};
use congru_synth::sequence::{Drift, Jump, Sequence};
use congru_synth::shapes;
use congru_track::{SensorView, Settings, Tracker};

const FRAMES: usize = 40;
const SPAN: f64 = 2.5;
const NOISE: f64 = 0.01;
const WITHIN_DEGREES: f64 = 2.0;
const KNOCK_FRAME: usize = 20;
const DRIFT: Drift = Drift {
    distance: 6.0,
    turns: 0.3,
    sway: 0.2,
};

fn model() -> OrientedCloud {
    shapes::lumpy_sphere_mesh(48)
        .unwrap()
        .sample_surface(Distance::new(0.05).unwrap())
        .unwrap()
}

fn errors(sequence: &Sequence) -> Vec<f64> {
    let model = model();
    let mut tracker = Tracker::new(
        "tracker",
        SensorView::new(model.clone()).unwrap(),
        Settings::for_part(SPAN).unwrap(),
    );
    sequence
        .shoot(model.cloud(), SPAN, 3)
        .unwrap()
        .iter()
        .map(|shot| {
            tracker.update(&shot.cloud).map_or(f64::INFINITY, |update| {
                update.pose.rotation_error_to(&shot.truth).to_degrees()
            })
        })
        .collect()
}

fn noisy() -> Sequence {
    Sequence {
        density: 0.3,
        noise: NOISE,
        clutter: 20,
        ..Sequence::still(FRAMES, DRIFT)
    }
}

#[test]
fn a_noisy_cluttered_sequence_is_followed() {
    let errors = errors(&noisy());
    let within = errors
        .iter()
        .filter(|error| **error < WITHIN_DEGREES)
        .count();
    assert!(within * 10 >= FRAMES * 9, "{errors:?}");
}

#[test]
fn a_knock_is_recovered_at_once() {
    let knocked = Sequence {
        jumps: vec![Jump {
            frame: KNOCK_FRAME,
            turn: [0.0, 1.5, 0.5],
            shift: [0.3, 0.0, 0.0],
        }],
        ..noisy()
    };
    let errors = errors(&knocked);
    let after = &errors[KNOCK_FRAME + 1..];
    assert!(
        after.iter().all(|error| *error < WITHIN_DEGREES),
        "{errors:?}"
    );
}
