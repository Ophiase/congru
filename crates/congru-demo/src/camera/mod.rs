//! Demo B: a machined part on a turntable under a simulated depth camera,
//! tracked through degraded frames.

mod conditions;
pub(crate) mod detect;
mod overlay;
mod room;
mod stage;
pub(crate) mod view;

use std::path::Path;

use congru_sim::{Turntable, bracket, turntable};
use congru_vision::Camera;

use crate::picture::palette::{MUTED, TEXT};
use crate::picture::{self, Tile, font};
use conditions::CONDITIONS;
use stage::{Stage, Step};

pub fn run(target: &Path) -> anyhow::Result<Vec<crate::Summary>> {
    let stage = Stage {
        part: bracket()?,
        turntable: turntable()?,
        surroundings: room::room()?,
        camera: Camera::with_field_of_view(240, 180, 60.0),
        path: Turntable {
            frames: 90,
            turns: 0.75,
            shake_degrees: 0.0,
            handheld: congru_synth::sequence::Handheld::default(),
        },
    };
    let runs: Vec<Vec<Step>> = CONDITIONS
        .iter()
        .map(|condition| stage.run(condition))
        .collect::<anyhow::Result<_>>()?;
    let frames: Vec<Vec<Tile>> = (0..stage.path.frames)
        .map(|frame| {
            CONDITIONS
                .iter()
                .zip(&runs)
                .map(|(condition, steps)| tile(&stage, condition.name, &steps[frame]))
                .collect()
        })
        .collect();
    picture::write(target, &frames, 3)?;
    Ok(CONDITIONS
        .iter()
        .zip(&runs)
        .map(|(condition, steps)| {
            crate::Summary::of(condition.name, steps.iter().map(error_degrees))
        })
        .collect())
}

fn tile(stage: &Stage, name: &str, step: &Step) -> Tile {
    let pose = step.update.as_ref().map(|update| update.pose);
    let mut picture = overlay::panel(stage, &step.depth, pose.as_ref());
    let error = error_degrees(step);
    let bottom = picture.height() - 22;
    font::write(&mut picture, 8, 8, name, MUTED);
    font::write(&mut picture, 8, bottom, &format!("{error:.1}°"), TEXT);
    Tile {
        picture,
        error_degrees: error,
    }
}

fn error_degrees(step: &Step) -> f64 {
    step.update.as_ref().map_or(f64::INFINITY, |update| {
        update.pose.rotation_error_to(&step.truth).to_degrees()
    })
}
