use std::time::Instant;

use congru::{Error, Frame, PointCloud};
use congru_scene::Scene;
use congru_sim::Degradation;
use congru_vision::{back_project, within, without_support};
use rayon::prelude::*;

use super::Trial;
use super::registration::perturbed;
use crate::catalog::{Method, Model};
use crate::suite::DETECTOR;

/// A consumer depth camera: noise, a few holes, flying pixels at edges.
const SENSOR: Degradation = Degradation {
    blur: 0,
    noise: 0.005,
    holes: 0.02,
    flying_pixels: true,
    lens: 0.0,
};
/// The detector's crop, around its guess of the target's centre, as a
/// share of the target's size.
const CROP: f64 = 0.75;
/// The plane under the target (a table, a road, a shelf board) is taken
/// away first, as any practical pipeline does: points within this share of
/// the target's size of the largest plane wider than the target.
const SUPPORT_TOLERANCE: f64 = 0.02;
const SUPPORT_WIDER: f64 = 1.5;

/// `method` finding the target of `scene` (whose model is `model`) from a
/// detector's guess, once per seed: the sensor's noise and the guess change
/// with the seed, the scene does not.
pub fn spot(
    method: &Method,
    model: &Model,
    scene: &Scene,
    seeds: std::ops::Range<u64>,
) -> Vec<Trial> {
    let exact = scene.render().depth;
    seeds
        .into_par_iter()
        .map(|seed| {
            trial(method, model, scene, &exact, seed).unwrap_or(super::registration::FAILED)
        })
        .collect()
}

fn trial(
    method: &Method,
    model: &Model,
    scene: &Scene,
    exact: &congru_vision::DepthImage,
    seed: u64,
) -> Result<Trial, Error> {
    let truth = scene.truth();
    let centre = model.target.cloud().centroid();
    let guess = perturbed(&truth, &centre, &DETECTOR, model.span, seed);
    let observed = back_project(&SENSOR.apply(exact, seed), &scene.camera);
    let points = without_support(
        &observed,
        SUPPORT_TOLERANCE * model.span,
        SUPPORT_WIDER * model.span,
    );
    let scan = PointCloud::new(within(&points, &guess.apply(&centre), CROP * model.span))?;
    let mut aligner = (method.build)(model)?;
    let started = Instant::now();
    let answer = aligner.align(&Frame {
        scan: &scan,
        guess: Some(guess),
    });
    let seconds = started.elapsed().as_secs_f64();
    Ok(answer.map_or(
        Trial {
            seconds,
            ..super::registration::FAILED
        },
        |estimate| Trial {
            rotation_degrees: estimate.pose.rotation_error_to(&truth).to_degrees(),
            translation: estimate.pose.translation_error_to(&truth) / model.span,
            trusted: estimate.trusted,
            seconds,
        },
    ))
}
