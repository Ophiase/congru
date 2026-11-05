//! Measures that tell the steps of demo B apart. The truth only scores here,
//! but for starting the tracker on purpose, to judge it apart from the search.

mod cases;
mod detection;
mod rank;

use std::io::Write;

use congru::Distance;
use congru_vision::{Finder, Observed};

use crate::camera::conditions::{CONDITIONS, Clutter};
use crate::camera::stage;
use crate::camera::stage::Step;

pub use cases::run as cases;
pub use detection::run as detection;
pub use rank::run as rank;

const PART_SPAN: f64 = 0.25;
const STEP_SHARE: f64 = 1.0 / 20.0;
const WITHIN_DEGREES: f64 = 2.0;
const APART_TURNS: [f64; 2] = [0.55, 1.3];

/// The whole pipeline on one condition, nothing from the truth; or, without
/// a name, each condition's tracker started from the truth.
pub fn run() -> anyhow::Result<()> {
    let stage = stage()?;
    let mut output = std::io::stdout().lock();
    match std::env::args().nth(2).as_deref() {
        Some("shares") => return shares(&stage, &mut output),
        Some("apart") => return apart(&stage, &mut output),
        Some(name) => return one_condition(&stage, name, &mut output),
        None => {}
    }
    for condition in &CONDITIONS {
        let mut seeded = Some(stage.sensed(condition, 0)?.1);
        let steps = stage.run_from(condition, |_, _| seeded.take())?;
        writeln!(output, "{}: {}", condition.name, tracking(&steps))?;
    }
    Ok(())
}

/// How much more the frame holds than the part, at the truth.
fn shares(stage: &stage::Stage, output: &mut impl Write) -> anyhow::Result<()> {
    for condition in &CONDITIONS {
        let shares = [0, 27, 45, 73].map(|frame| {
            stage
                .crowding_at_truth(condition, frame)
                .unwrap_or(f64::NAN)
        });
        writeln!(output, "{}: {shares:.3?}", condition.name)?;
    }
    Ok(())
}

/// The whole pipeline on one condition, frame by frame.
fn one_condition(stage: &stage::Stage, name: &str, output: &mut impl Write) -> anyhow::Result<()> {
    let condition = CONDITIONS
        .iter()
        .find(|condition| condition.name == name)
        .ok_or_else(|| anyhow::anyhow!("no condition {name}"))?;
    let began = std::time::Instant::now();
    let steps = stage.run(condition)?;
    let seconds = began.elapsed().as_secs_f64();
    writeln!(output, "{name}: {}, in {seconds:.0} s", tracking(&steps))?;
    let finder = Finder::new(&stage.part, Distance::new(PART_SPAN * STEP_SHARE)?)?;
    let tolerance = PART_SPAN * STEP_SHARE / 2.0;
    for (frame, step) in steps.iter().enumerate() {
        let line = step.update.as_ref().map_or("nothing".to_owned(), |update| {
            let seen = Observed::new(step.depth.clone(), &stage.camera, tolerance);
            format!(
                "{:.1}° confirms {:.2} at truth {:.2} {:?} trusted {} inliers {:.2} scan {}",
                update.pose.rotation_error_to(&step.truth).to_degrees(),
                finder.confirmed(&seen, &stage.camera, &update.pose),
                finder.confirmed(&seen, &stage.camera, &step.truth),
                update.phase,
                update.trusted,
                update.alignment.inlier_fraction,
                step.scan.len(),
            )
        });
        writeln!(output, "{frame:2}: {line}")?;
    }
    Ok(())
}

/// The same pipeline on a path no choice was made on: other turns, so other
/// views of the part at every frame.
fn apart(stage: &stage::Stage, output: &mut impl Write) -> anyhow::Result<()> {
    for turns in APART_TURNS {
        let other = stage.turning(turns);
        for condition in CONDITIONS
            .iter()
            .filter(|condition| condition.clutter == Clutter::Room)
        {
            let steps = other.run(condition)?;
            writeln!(
                output,
                "{turns} turns, {}: {}",
                condition.name,
                tracking(&steps)
            )?;
        }
    }
    Ok(())
}

fn tracking(steps: &[Step]) -> String {
    let errors: Vec<f64> = steps
        .iter()
        .map(|step| {
            step.update.as_ref().map_or(f64::INFINITY, |update| {
                update.pose.rotation_error_to(&step.truth).to_degrees()
            })
        })
        .collect();
    let mut sorted = errors.clone();
    sorted.sort_by(f64::total_cmp);
    let off: Vec<usize> = (0..errors.len())
        .filter(|frame| errors[*frame] >= WITHIN_DEGREES)
        .collect();
    format!(
        "median {:.2}°, {} of {} within {WITHIN_DEGREES}°, first off {:?}",
        sorted[sorted.len() / 2],
        errors.len() - off.len(),
        errors.len(),
        off.first(),
    )
}
