use std::path::Path;

use anyhow::{Context, bail};
use congru::{Distance, Frame, RigidTransform};
use serde::Serialize;

use crate::cli::Align;
use crate::files::read;
use crate::model::{Model, thinned};
use crate::report::Observed;

#[derive(Debug, Serialize)]
pub struct Answer {
    method: &'static str,
    /// Takes the source onto the target, row-major.
    transform: [f64; 16],
    trusted: bool,
    observability: Option<Observed>,
}

/// # Errors
/// When a file cannot be read, the guess is not a rigid transform, or the
/// method finds no pose at all.
pub fn run(arguments: &Align) -> anyhow::Result<Answer> {
    let spacing = arguments
        .voxel
        .map(Distance::new)
        .transpose()
        .context("--voxel must be finite and positive")?;
    let source = thinned(&read(&arguments.source)?.points, spacing);
    let model = Model::new(&read(&arguments.target)?, spacing)?;
    let guess = arguments.guess.as_deref().map(parse_guess).transpose()?;
    let mut aligner = arguments.method.build(&model)?;
    let estimate = aligner.align(&Frame {
        scan: &source,
        guess: guess.map(|guess| guess.inverse()),
    })?;
    Ok(Answer {
        method: arguments.method.name(),
        transform: estimate.pose.inverse().to_row_major(),
        trusted: estimate.trusted,
        observability: estimate.observability.as_ref().map(Observed::from),
    })
}

/// Sixteen numbers separated by spaces or commas, row-major, or the path of
/// a file holding them.
fn parse_guess(text: &str) -> anyhow::Result<RigidTransform> {
    let path = Path::new(text);
    let content = if path.is_file() {
        std::fs::read_to_string(path).with_context(|| format!("cannot read {text}"))?
    } else {
        text.to_owned()
    };
    let numbers = content
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|word| !word.is_empty())
        .map(str::parse)
        .collect::<Result<Vec<f64>, _>>()
        .context("--guess holds something that is not a number")?;
    let Ok(entries) = <[f64; 16]>::try_from(numbers.as_slice()) else {
        bail!("--guess needs 16 numbers, got {}", numbers.len());
    };
    RigidTransform::from_row_major(&entries).context("--guess is not a rigid transform")
}
