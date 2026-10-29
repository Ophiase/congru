use std::time::Instant;

use congru::{Error, Frame, Point, PointCloud, RigidTransform};
use congru_synth::{MotionBounds, Scenario};
use nalgebra::{UnitQuaternion, Vector3};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};
use rayon::prelude::*;

use super::Trial;
use crate::catalog::{Method, Model};
use crate::suite::Situation;

/// `method` on `model` in `situation`, once per seed.
pub fn register(
    method: &Method,
    model: &Model,
    situation: &Situation,
    seeds: std::ops::Range<u64>,
) -> Vec<Trial> {
    seeds
        .into_par_iter()
        .map(|seed| trial(method, model, situation, seed).unwrap_or(FAILED))
        .collect()
}

const FAILED: Trial = Trial {
    rotation_degrees: f64::INFINITY,
    translation: f64::INFINITY,
    trusted: false,
    seconds: 0.0,
};

/// What a method is handed in one trial, and the truth it is scored on.
#[derive(Debug, Clone)]
pub struct Scene {
    pub observed: PointCloud,
    /// How many of the observed points are outliers; they come last.
    pub outliers: usize,
    pub truth: RigidTransform,
    pub guess: RigidTransform,
}

/// The frame of `situation` on `model` for one seed.
///
/// # Errors
/// When the situation leaves nothing to observe.
pub fn scene(model: &Model, situation: &Situation, seed: u64) -> Result<Scene, Error> {
    let sampled = thinned(model, situation.density)?;
    let corruption = situation.corruption(model.span, sampled.count())?;
    let scenario = Scenario::generate(
        &sampled,
        &corruption,
        &MotionBounds::any_rotation(model.span),
        seed,
    )?;
    let guess = perturbed(
        &scenario.truth,
        &sampled.centroid(),
        situation,
        model.span,
        seed,
    );
    Ok(Scene {
        observed: scenario.observed,
        outliers: scenario.outliers,
        truth: scenario.truth,
        guess,
    })
}

fn trial(method: &Method, model: &Model, situation: &Situation, seed: u64) -> Result<Trial, Error> {
    let scenario = scene(model, situation, seed)?;
    let guess = scenario.guess;
    let mut aligner = (method.build)(model)?;
    let started = Instant::now();
    let answer = aligner.align(&Frame {
        scan: &scenario.observed,
        guess: Some(guess),
    });
    let seconds = started.elapsed().as_secs_f64();
    Ok(
        answer.map_or(Trial { seconds, ..FAILED }, |estimate| Trial {
            rotation_degrees: estimate
                .pose
                .rotation_error_to(&scenario.truth)
                .to_degrees(),
            translation: estimate.pose.translation_error_to(&scenario.truth) / model.span,
            trusted: estimate.trusted,
            seconds,
        }),
    )
}

fn thinned(model: &Model, density: f64) -> Result<PointCloud, Error> {
    let stride = super::share_of(1, 1.0 / density).max(1);
    PointCloud::new(
        model
            .observed
            .cloud()
            .points()
            .iter()
            .step_by(stride)
            .copied()
            .collect::<Vec<Point>>(),
    )
}

/// The truth moved by a turn of the situation's angle about the model's
/// centre, on a random axis, and a shift of its size in a random direction.
fn perturbed(
    truth: &RigidTransform,
    centre: &Point,
    situation: &Situation,
    span: f64,
    seed: u64,
) -> RigidTransform {
    let mut random = ChaCha8Rng::seed_from_u64(seed ^ 0x9e37_79b9);
    let mut direction = || Vector3::from_fn(|_, _| random.random_range(-1.0..1.0)).normalize();
    let turn = UnitQuaternion::from_scaled_axis(direction() * situation.turn.to_radians());
    let shift = direction() * situation.shift * span;
    let about_centre =
        RigidTransform::from_parts(turn, centre.coords - turn * centre.coords + shift);
    about_centre.then(truth)
}
