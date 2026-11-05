//! The search alone, frame by frame, and on a frame without the part: a
//! threshold is only worth what it separates.

use std::io::Write;
use std::time::Instant;

use congru::{Distance, RigidTransform};
use congru_vision::{Finder, Observed, render};

use super::{PART_SPAN, STEP_SHARE};
use crate::camera::conditions::{CONDITIONS, Condition};
use crate::camera::detect::region_of_interest;
use crate::camera::stage;

const FRAMES: [usize; 8] = [0, 7, 20, 27, 40, 51, 73, 86];
const WITHIN_DEGREES: f64 = 5.0;
const WITHIN_METRES: f64 = 0.02;

/// One frame, searched only inside its region of interest.
fn in_region(name: &str, turns: f64, frames: &[usize]) -> anyhow::Result<()> {
    let stage = stage()?.turning(turns);
    let tight = PART_SPAN * STEP_SHARE / 2.0;
    let finder = Finder::new(&stage.part, Distance::new(PART_SPAN * STEP_SHARE)?)?;
    let condition = CONDITIONS
        .iter()
        .find(|condition| condition.name == name)
        .ok_or_else(|| anyhow::anyhow!("no condition {name}"))?;
    let mut output = std::io::stdout().lock();
    for frame in frames {
        let (depth, truth) = stage.sensed(condition, *frame)?;
        let region = region_of_interest(&depth, &stage.camera, &stage.part, &truth, *frame as u64);
        let seen = Observed::new(region.clone(), &stage.camera, tight);
        let found = finder.find_in(&depth, &region, &stage.camera, f64::NEG_INFINITY);
        let line = found.map_or("nothing".to_owned(), |found| {
            format!(
                "{:.1}° {:.0} mm, score {:.1} confirms {:.2}",
                found.pose.rotation_error_to(&truth).to_degrees(),
                1e3 * found.pose.translation_error_to(&truth),
                1e4 * finder.score(&seen, &stage.camera, &found.pose, tight),
                found.agreement,
            )
        });
        let at_truth = 1e4 * finder.score(&seen, &stage.camera, &truth, tight);
        writeln!(
            output,
            "frame {frame}: truth scores {at_truth:.1}; found {line}"
        )?;
    }
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(2).collect();
    if let [name, turns, frames @ ..] = &arguments[..] {
        let frames: Vec<usize> = frames
            .iter()
            .filter_map(|frame| frame.parse().ok())
            .collect();
        return in_region(name, turns.parse()?, &frames);
    }
    let stage = stage()?;
    let finder = Finder::new(&stage.part, Distance::new(PART_SPAN * STEP_SHARE)?)?;
    let mut output = std::io::stdout().lock();
    for condition in CONDITIONS.iter().filter(|condition| {
        ["room", "in a room", "all effects", "sensor"].contains(&condition.name)
    }) {
        writeln!(output, "## {}", condition.name)?;
        let mut right = 0;
        for frame in FRAMES {
            let (line, was_right) = one_frame(&stage, &finder, condition, frame)?;
            right += usize::from(was_right);
            writeln!(output, "frame {frame:2}: {line}")?;
        }
        writeln!(output, "right: {right} of {}", FRAMES.len())?;
    }
    let pose = RigidTransform::identity();
    let room = render(
        &stage.camera,
        &[
            (&stage.turntable, stage.path.pose(0)),
            (&stage.surroundings, pose),
        ],
    )
    .depth;
    let absent = finder.find(&room, &stage.camera, f64::NEG_INFINITY);
    writeln!(
        output,
        "without the part: confirms {:?}",
        absent.map(|found| found.agreement)
    )?;
    Ok(())
}

fn one_frame(
    stage: &stage::Stage,
    finder: &Finder,
    condition: &Condition,
    frame: usize,
) -> anyhow::Result<(String, bool)> {
    let (depth, truth) = stage.sensed(condition, frame)?;
    let began = Instant::now();
    let found = finder.find(&depth, &stage.camera, f64::NEG_INFINITY);
    let seconds = began.elapsed().as_secs_f64();
    Ok(found.map_or(("nothing".to_owned(), false), |found| {
        let degrees = found.pose.rotation_error_to(&truth).to_degrees();
        let metres = found.pose.translation_error_to(&truth);
        (
            format!(
                "{degrees:.1}° {metres:.3} m, confirms {:.2} in {seconds:.1} s",
                found.agreement
            ),
            degrees < WITHIN_DEGREES && metres < WITHIN_METRES,
        )
    }))
}
