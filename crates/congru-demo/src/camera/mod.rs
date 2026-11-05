//! Demo B: a machined part on a turntable under a simulated depth camera,
//! tracked through degraded frames.

pub(crate) mod conditions;
pub(crate) mod detect;
pub(crate) mod overlay;
mod room;
pub(crate) mod stage;
pub(crate) mod view;

use std::path::{Path, PathBuf};

use congru_sim::{Solids, Turntable, bracket, turntable};
use congru_vision::Camera;

use crate::picture::palette::{MUTED, TEXT};
use crate::picture::{self, Tile, font};
use conditions::CONDITIONS;
use stage::{Stage, Step};

/// The turntable under its camera, as every condition sees it.
pub(crate) fn stage() -> anyhow::Result<Stage> {
    Ok(Stage {
        part: bracket()?,
        turntable: turntable()?,
        surroundings: room::room()?,
        occluder: Solids::default()
            .cylinder([0.0, 0.0, -0.04], 0.035, 0.08, 16)
            .build()?,
        camera: Camera::with_field_of_view(160, 120, 60.0),
        path: Turntable {
            frames: 90,
            turns: 0.75,
            shake_degrees: 0.0,
            handheld: congru_synth::sequence::Handheld::default(),
            knock_degrees: 0.0,
        },
    })
}

pub fn run(target: &Path) -> anyhow::Result<Vec<crate::Summary>> {
    let stage = stage()?;
    let runs: Vec<Vec<Step>> = CONDITIONS
        .iter()
        .map(|condition| stage.run(condition))
        .collect::<anyhow::Result<_>>()?;
    write(&stage, &runs, target)?;
    Ok(CONDITIONS
        .iter()
        .zip(&runs)
        .map(|(condition, steps)| {
            crate::Summary::of(condition.name, steps.iter().map(error_degrees))
        })
        .collect())
}

/// A file next to `target`, named after the condition.
/// The first two conditions together at `target`, each other one beside it.
fn write(stage: &Stage, runs: &[Vec<Step>], target: &Path) -> anyhow::Result<()> {
    let rows: Vec<Vec<Vec<Tile>>> = CONDITIONS
        .iter()
        .zip(runs)
        .map(|(condition, steps)| {
            steps
                .iter()
                .map(|step| tiles(stage, condition.name, step).to_vec())
                .collect()
        })
        .collect();
    let together: Vec<Vec<Tile>> = rows[0]
        .iter()
        .zip(&rows[1])
        .map(|(first, second)| [first.clone(), second.clone()].concat())
        .collect();
    picture::write(target, &together, 3)?;
    for (condition, frames) in CONDITIONS.iter().zip(&rows).skip(2) {
        picture::write(&beside(target, condition.name), frames, 3)?;
    }
    Ok(())
}

fn beside(target: &Path, name: &str) -> PathBuf {
    let stem = target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("tracking");
    target.with_file_name(format!("{stem}-{}.gif", name.replace(' ', "-")))
}

fn tiles(stage: &Stage, name: &str, step: &Step) -> [Tile; 3] {
    let pose = step.update.as_ref().map(|update| update.pose);
    let error = error_degrees(step);
    let [image, mut depth, cloud] = overlay::panels(stage, step, pose.as_ref());
    let bottom = depth.height() - 22;
    font::write(&mut depth, 8, bottom, &format!("{error:.1}°"), TEXT);
    [
        (name.to_owned(), image),
        ("depth".to_owned(), depth),
        ("points".to_owned(), cloud),
    ]
    .map(|(label, mut picture)| {
        font::write(&mut picture, 8, 8, &label, MUTED);
        Tile {
            picture,
            error_degrees: error,
        }
    })
}

fn error_degrees(step: &Step) -> f64 {
    step.update.as_ref().map_or(f64::INFINITY, |update| {
        update.pose.rotation_error_to(&step.truth).to_degrees()
    })
}
