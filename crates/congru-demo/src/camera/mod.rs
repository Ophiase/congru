//! Demo B: a machined part on a turntable under a simulated depth camera,
//! tracked through degraded frames.

mod conditions;
mod overlay;
mod stage;
mod tracking;

use std::path::Path;

use congru_sim::{Turntable, bracket, turntable};
use congru_vision::Camera;

use crate::picture::{self, Tile};
use conditions::CONDITIONS;
use stage::{Stage, Step};

pub fn run(target: &Path) -> anyhow::Result<Vec<crate::Summary>> {
    let stage = Stage {
        part: bracket()?,
        turntable: turntable()?,
        camera: Camera::with_field_of_view(240, 180, 60.0),
        path: Turntable {
            frames: 90,
            turns: 0.75,
        },
    };
    let runs: Vec<Vec<Step>> = CONDITIONS
        .iter()
        .map(|condition| stage.run(condition))
        .collect::<anyhow::Result<_>>()?;
    let frames: Vec<Vec<Tile>> = (0..stage.path.frames)
        .map(|frame| {
            runs.iter()
                .map(|steps| tile(&stage, &steps[frame]))
                .collect()
        })
        .collect();
    picture::write(target, &frames, 2)?;
    Ok(CONDITIONS
        .iter()
        .zip(&runs)
        .map(|(condition, steps)| {
            crate::Summary::of(condition.name, steps.iter().map(error_degrees))
        })
        .collect())
}

fn tile(stage: &Stage, step: &Step) -> Tile {
    let pose = step.estimate.as_ref().map(|estimate| estimate.pose);
    Tile {
        picture: overlay::panel(stage, &step.depth, pose.as_ref()),
        error_degrees: error_degrees(step),
    }
}

fn error_degrees(step: &Step) -> f64 {
    step.estimate.as_ref().map_or(f64::INFINITY, |estimate| {
        estimate.pose.rotation_error_to(&step.truth).to_degrees()
    })
}
