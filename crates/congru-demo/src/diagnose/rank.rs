//! Does a score put the truth first? Wrong poses are improved by a local
//! search on the score itself, since a real search will reach them.

use std::io::Write;

use congru::{Distance, RigidTransform};
use congru_vision::{Camera, Finder, Observed, back_project};
use nalgebra::{UnitQuaternion, Vector3};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};
use rayon::prelude::*;

use super::{PART_SPAN, STEP_SHARE};
use crate::camera::conditions::CONDITIONS;
use crate::camera::stage;

const RANDOM_STARTS: usize = 100;
const CLIMB_STEPS: usize = 60;
/// The frames a score was chosen on, and the ones kept apart to check it.
const TUNING_FRAMES: [usize; 4] = [0, 20, 40, 60];
const HELD_OUT_FRAMES: [usize; 6] = [7, 27, 34, 51, 73, 86];
/// A pose this close to the truth is not a wrong pose.
const NEAR_DEGREES: f64 = 5.0;
const NEAR_METRES: f64 = 0.02;
const REACH: f64 = 0.15;
/// The scale that makes a score of areas readable.
const READABLE: f64 = 1e4;

/// The scores under trial.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Information(f64),
}

const MODES: [(Mode, &str); 2] = [
    (Mode::Information(1.0), "info step "),
    (Mode::Information(0.5), "info half "),
];
/// The best-scored starts that are improved, as a real search would.
const CLIMBED: usize = 64;

#[derive(Debug, Clone, Copy)]
struct Scorer<'scene> {
    finder: &'scene Finder,
    camera: Camera,
    seen: &'scene [Observed; 2],
    step: f64,
    mode: Mode,
}

impl Scorer<'_> {
    fn score(&self, pose: &RigidTransform) -> f64 {
        match self.mode {
            Mode::Information(share) => {
                let tolerance = self.step * share;
                let seen = &self.seen[usize::from(share < 1.0)];
                READABLE * self.finder.score(seen, &self.camera, pose, tolerance)
            }
        }
    }

    /// Random steps kept when they score higher, shrinking as they go.
    fn climb(&self, start: &RigidTransform, seed: u64) -> (RigidTransform, f64) {
        let mut random = ChaCha8Rng::seed_from_u64(seed);
        let (mut pose, mut best) = (*start, self.score(start));
        for step in 0..CLIMB_STEPS {
            let shrink = 1.0 - f64::from(u32::try_from(step).unwrap_or(0)) / 90.0;
            let trial = pose.then(&jolt(&mut random, 0.15 * shrink, 0.02 * shrink));
            let value = self.score(&trial);
            if value > best {
                (pose, best) = (trial, value);
            }
        }
        (pose, best)
    }
}

fn jolt(random: &mut ChaCha8Rng, angle: f64, shift: f64) -> RigidTransform {
    let mut draw = |size: f64| size * (random.random::<f64>() * 2.0 - 1.0);
    RigidTransform::from_parts(
        UnitQuaternion::from_scaled_axis(Vector3::new(draw(angle), draw(angle), draw(angle))),
        [draw(shift), draw(shift), draw(shift)],
    )
}

fn random_pose(random: &mut ChaCha8Rng, points: &[congru::Point]) -> RigidTransform {
    let turn = UnitQuaternion::from_scaled_axis(
        Vector3::new(
            random.random::<f64>() - 0.5,
            random.random::<f64>() - 0.5,
            random.random::<f64>() - 0.5,
        ) * 6.0,
    );
    let anchor = points[random.random_range(0..points.len())];
    let deeper = random.random::<f64>() * REACH;
    RigidTransform::from_parts(turn, [anchor.x, anchor.y, anchor.z + deeper])
}

pub fn run() -> anyhow::Result<()> {
    let stage = stage()?;
    let step = PART_SPAN * STEP_SHARE;
    let finder = Finder::new(&stage.part, Distance::new(step)?)?;
    let mut output = std::io::stdout().lock();
    let sets = [
        ("tuning", &TUNING_FRAMES[..]),
        ("held out", &HELD_OUT_FRAMES[..]),
    ];
    for (set, frames) in sets {
        for condition in CONDITIONS
            .iter()
            .filter(|condition| ["room", "in a room", "sensor"].contains(&condition.name))
        {
            writeln!(output, "## {set}, {}", condition.name)?;
            for frame in frames {
                let (depth, truth) = stage.sensed(condition, *frame)?;
                let points = back_project(&depth, &stage.camera);
                let seen = [
                    Observed::new(depth.clone(), &stage.camera, step),
                    Observed::new(depth.clone(), &stage.camera, step / 2.0),
                ];
                let mut random = ChaCha8Rng::seed_from_u64(*frame as u64 + 1000);
                let mut starts = finder.proposals(&depth, &stage.camera);
                let voted = starts.len();
                starts.extend((0..RANDOM_STARTS).map(|_| random_pose(&mut random, &points)));
                for (mode, name) in MODES {
                    let scorer = Scorer {
                        finder: &finder,
                        camera: stage.camera,
                        seen: &seen,
                        step,
                        mode,
                    };
                    let line = compare(&scorer, &starts, &truth);
                    writeln!(output, "frame {frame:2} ({voted} votes) {name}: {line}")?;
                }
            }
        }
    }
    Ok(())
}

fn compare(scorer: &Scorer<'_>, starts: &[RigidTransform], truth: &RigidTransform) -> String {
    let mut ranked: Vec<(f64, RigidTransform)> = starts
        .par_iter()
        .map(|start| (scorer.score(start), *start))
        .collect();
    ranked.sort_by(|left, right| right.0.total_cmp(&left.0));
    let climbed: Vec<(RigidTransform, f64)> = ranked
        .par_iter()
        .take(CLIMBED)
        .enumerate()
        .map(|(index, (_, start))| scorer.climb(start, index as u64))
        .collect();
    let wrong = |pose: &RigidTransform| {
        pose.rotation_error_to(truth).to_degrees() > NEAR_DEGREES
            || pose.translation_error_to(truth) > NEAR_METRES
    };
    let best_wrong = climbed
        .iter()
        .filter(|(pose, _)| wrong(pose))
        .max_by(|left, right| left.1.total_cmp(&right.1));
    let reached = climbed.iter().filter(|(pose, _)| !wrong(pose)).count();
    let (_, truth_climbed) = scorer.climb(truth, 7);
    let Some((pose, score)) = best_wrong else {
        return "no wrong pose".to_owned();
    };
    format!(
        "truth {:.2} climbed {truth_climbed:.2}; wrong {score:.2} at {:.0}° {:.3} m; {reached} near; {}",
        scorer.score(truth),
        pose.rotation_error_to(truth).to_degrees(),
        pose.translation_error_to(truth),
        if truth_climbed > *score {
            "TRUTH"
        } else {
            "wrong"
        },
    )
}
