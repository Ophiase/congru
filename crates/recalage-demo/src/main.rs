//! A simulated robot tracking a machined part through a degraded depth
//! camera, under several conditions, written as one animation.

mod conditions;
mod picture;
mod sequence;
mod tracking;

use std::io::Write;
use std::path::PathBuf;

use recalage_sim::{Turntable, bracket, turntable};
use recalage_vision::Camera;

use crate::conditions::CONDITIONS;
use crate::picture::Tile;
use crate::sequence::{Stage, Step};

fn main() -> anyhow::Result<()> {
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
    let target = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("docs/figures/tracking.gif"), PathBuf::from);
    picture::write(&target, &frames, 2)?;
    report(&runs)?;
    Ok(())
}

fn tile(stage: &Stage, step: &Step) -> Tile {
    let pose = step.estimate.as_ref().map(|estimate| estimate.pose);
    Tile {
        picture: picture::panel(stage, &step.depth, pose.as_ref()),
        error_degrees: pose.map_or(f64::INFINITY, |pose| {
            pose.rotation_error_to(&step.truth).to_degrees()
        }),
    }
}

fn report(runs: &[Vec<Step>]) -> anyhow::Result<()> {
    let mut output = std::io::stdout().lock();
    for (condition, steps) in CONDITIONS.iter().zip(runs) {
        let mut errors: Vec<f64> = steps
            .iter()
            .map(|step| {
                step.estimate.as_ref().map_or(f64::INFINITY, |estimate| {
                    estimate.pose.rotation_error_to(&step.truth).to_degrees()
                })
            })
            .collect();
        if std::env::var_os("RECALAGE_TRACE").is_some() {
            let trace: Vec<String> = errors.iter().map(|error| format!("{error:.1}")).collect();
            writeln!(output, "{}", trace.join(" "))?;
        }
        errors.sort_by(f64::total_cmp);
        let within = errors.iter().filter(|error| **error < 2.0).count();
        writeln!(
            output,
            "{:32} median {:5.2}°  90th {:6.2}°  within 2°: {within}/{}",
            condition.name,
            errors[errors.len() / 2],
            errors[errors.len() * 9 / 10],
            errors.len()
        )?;
    }
    Ok(())
}
