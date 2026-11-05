//! The search alone, frame by frame, and on a frame without the part: a
//! threshold is only worth what it separates.

use std::io::Write;
use std::time::Instant;

use congru::{Distance, RigidTransform};
use congru_vision::{Finder, Observed, render};

use super::{PART_SPAN, STEP_SHARE};
use crate::camera::conditions::CONDITIONS;
use crate::camera::stage;

const FRAMES: [usize; 8] = [0, 7, 20, 27, 40, 51, 73, 86];
const WITHIN_DEGREES: f64 = 5.0;
const WITHIN_METRES: f64 = 0.02;

/// One frame only: is the right pose among the votes, and where does it rank?
fn one_frame(name: &str, frame: usize) -> anyhow::Result<()> {
    let stage = stage()?;
    let step = PART_SPAN * STEP_SHARE;
    let finder = Finder::new(&stage.part, Distance::new(step)?)?;
    let condition = CONDITIONS
        .iter()
        .find(|condition| condition.name == name)
        .ok_or_else(|| anyhow::anyhow!("no such condition"))?;
    let (depth, truth) = stage.sensed(condition, frame)?;
    let loose = Observed::new(depth.clone(), &stage.camera, step);
    let mut output = std::io::stdout().lock();
    let near = |pose: &RigidTransform, degrees: f64, metres: f64| {
        pose.rotation_error_to(&truth).to_degrees() < degrees
            && pose.translation_error_to(&truth) < metres
    };
    let votes = finder.raw_votes(&depth, &stage.camera, 4);
    for iterations in [0, 2, 4] {
        let began = Instant::now();
        let brought: Vec<RigidTransform> = votes
            .iter()
            .map(|pose| finder.refined_by(&depth, &stage.camera, pose, iterations))
            .collect();
        let refining = began.elapsed().as_secs_f64();
        let mut scored: Vec<(f64, RigidTransform)> = brought
            .iter()
            .map(|pose| (finder.score(&loose, &stage.camera, pose, step), *pose))
            .collect();
        scored.sort_by(|left, right| right.0.total_cmp(&left.0));
        let close: Vec<usize> = scored
            .iter()
            .enumerate()
            .filter(|(_, (_, pose))| near(pose, 10.0, 0.02))
            .map(|(rank, _)| rank)
            .take(6)
            .collect();
        writeln!(
            output,
            "{iterations} iterations in {refining:.1} s, {} votes: ranks within 10°, 2 cm: {close:?}",
            votes.len(),
        )?;
    }
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(2).collect();
    if let [name, frame] = &arguments[..] {
        return one_frame(name, frame.parse()?);
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
            let (depth, truth) = stage.sensed(condition, frame)?;
            let began = Instant::now();
            let found = finder.find(&depth, &stage.camera, f64::NEG_INFINITY);
            let seconds = began.elapsed().as_secs_f64();
            let line = found.map_or("nothing".to_owned(), |found| {
                let degrees = found.pose.rotation_error_to(&truth).to_degrees();
                let metres = found.pose.translation_error_to(&truth);
                if degrees < WITHIN_DEGREES && metres < WITHIN_METRES {
                    right += 1;
                }
                format!(
                    "{degrees:.1}° {metres:.3} m, confirms {:.2}",
                    found.agreement
                )
            });
            writeln!(output, "frame {frame:2}: {line} in {seconds:.1} s")?;
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
