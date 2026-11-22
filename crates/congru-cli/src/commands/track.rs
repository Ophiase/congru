use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use congru_track::{Phase, SensorView, Settings, Tracker};
use serde::Serialize;

use crate::cli::Track;
use crate::files::{is_point_file, read};
use crate::model::Model;
use crate::report::emit;

#[derive(Debug, Serialize)]
struct Line {
    frame: usize,
    file: String,
    /// Takes the model into the frame's coordinates, row-major.
    transform: Option<[f64; 16]>,
    phase: Option<&'static str>,
    trusted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// # Errors
/// When the model or the folder cannot be read; a frame that fails is a
/// line with an `error`, and the next frame starts with a search.
pub fn run(arguments: &Track, output: &mut dyn Write) -> anyhow::Result<()> {
    let model = Model::new(&read(&arguments.model)?, None)?;
    let span = arguments.span.unwrap_or(model.span);
    let mut tracker = Tracker::new(
        "tracker",
        SensorView::new(model.surface)?,
        Settings::for_part(span).context("--span must be finite and positive")?,
    );
    for (frame, path) in frames(&arguments.frames)?.iter().enumerate() {
        let file = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let update = read(path).and_then(|shape| Ok(tracker.update(&shape.points)?));
        let line = match update {
            Ok(update) => Line {
                frame,
                file,
                transform: Some(update.pose.to_row_major()),
                phase: Some(match update.phase {
                    Phase::Follow => "follow",
                    Phase::Find => "find",
                }),
                trusted: update.trusted,
                error: None,
            },
            Err(error) => Line {
                frame,
                file,
                transform: None,
                phase: None,
                trusted: false,
                error: Some(format!("{error:#}")),
            },
        };
        emit(output, &line)?;
    }
    Ok(())
}

fn frames(folder: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(folder)
        .with_context(|| format!("cannot list {}", folder.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|path| path.is_file() && is_point_file(path));
    paths.sort();
    if paths.is_empty() {
        bail!("{} holds no .xyz or .ply frame", folder.display());
    }
    Ok(paths)
}
