use std::time::Instant;

use congru::{Error, Frame, RigidTransform};
use rayon::prelude::*;

use super::Trial;
use crate::catalog::{Method, Model};
use crate::suite::Sequence;

const FIRST_GUESS_DEGREES: f64 = 10.0;

/// `method` on `model` through `sequence`, once per seed; one trial per
/// frame. Every method gets the same first guess, a pose 10° off as from a
/// detector; later, a one-shot method starts from its own last answer.
pub fn track(
    method: &Method,
    model: &Model,
    sequence: &Sequence,
    seeds: std::ops::Range<u64>,
) -> Vec<Trial> {
    seeds
        .into_par_iter()
        .flat_map_iter(|seed| run(method, model, sequence, seed).unwrap_or_default())
        .collect()
}

fn run(
    method: &Method,
    model: &Model,
    sequence: &Sequence,
    seed: u64,
) -> Result<Vec<Trial>, Error> {
    let shots = sequence
        .frames
        .shoot(model.target.cloud(), model.span, seed)?;
    let mut aligner = (method.build)(model)?;
    let mut guess = shots.first().map(|shot| nudged(&shot.truth));
    Ok(shots
        .iter()
        .map(|shot| {
            let started = Instant::now();
            let answer = aligner.align(&Frame {
                scan: &shot.cloud,
                guess,
            });
            let seconds = started.elapsed().as_secs_f64();
            match answer {
                Ok(estimate) => {
                    guess = Some(estimate.pose);
                    Trial {
                        rotation_degrees: estimate.pose.rotation_error_to(&shot.truth).to_degrees(),
                        translation: estimate.pose.translation_error_to(&shot.truth) / model.span,
                        trusted: estimate.trusted,
                        seconds,
                    }
                }
                Err(_) => Trial {
                    rotation_degrees: f64::INFINITY,
                    translation: f64::INFINITY,
                    trusted: false,
                    seconds,
                },
            }
        })
        .collect())
}

fn nudged(truth: &RigidTransform) -> RigidTransform {
    let turn = RigidTransform::from_axis_angle(
        nalgebra::Vector3::new(1.0, 1.0, 0.0).normalize() * FIRST_GUESS_DEGREES.to_radians(),
        [0.0; 3],
    );
    turn.then(truth)
}
