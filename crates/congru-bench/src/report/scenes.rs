use std::fmt::Write;
use std::path::Path;

use congru::{Point, RigidTransform};
use congru_figures::{Figure, Ink, View, sheet};
use nalgebra::Vector3;

use crate::catalog::Model;
use crate::measure::scene;
use crate::suite::{SITUATIONS, Situation};

const SEED: u64 = 1_000_000;
const COLUMNS: usize = 4;
const CELL: [f64; 2] = [300.0, 334.0];
const DRAWN_MODEL_POINTS: usize = 300;
const DRAWN_SCAN_POINTS: usize = 400;

/// Draws every situation on every model as the methods receive it, into
/// `folder`, and returns the page that shows them.
///
/// # Errors
/// When a file cannot be written, or a situation leaves nothing to draw.
pub fn scenes(models: &[Model], folder: &Path) -> anyhow::Result<String> {
    std::fs::create_dir_all(folder)?;
    let mut page = String::new();
    writeln!(page, "# Scenes\n")?;
    writeln!(
        page,
        "Every situation of the [protocol](protocol.md), drawn exactly as the\n\
         methods receive it, for the first evaluation seed: the scan in blue,\n\
         its outliers in green, the model at the starting guess in red. A\n\
         method must bring the red onto the blue. Written by\n\
         `just bench-scenes`.\n"
    )?;
    for model in models {
        writeln!(page, "## {}\n\n{}.\n", model.name, model.summary)?;
        let figures = SITUATIONS
            .iter()
            .map(|situation| draw(model, situation))
            .collect::<anyhow::Result<Vec<String>>>()?;
        let name = format!("{}.svg", model.name);
        std::fs::write(folder.join(&name), sheet(&figures, COLUMNS, CELL))?;
        writeln!(
            page,
            "![Every situation on the {0}](scenes/{name})\n",
            model.name
        )?;
    }
    writeln!(
        page,
        "## Sequences\n\n![The tracking sequences](../figures/clouds.gif)\n"
    )?;
    writeln!(
        page,
        "The tracking sequences, as [demo A](../demos.md) shows them on the bracket."
    )?;
    Ok(page)
}

fn draw(model: &Model, situation: &Situation) -> anyhow::Result<String> {
    let frame = scene(model, situation, SEED)?;
    let stride = model
        .observed
        .cloud()
        .count()
        .div_ceil(DRAWN_MODEL_POINTS)
        .max(1);
    let guessed: Vec<Point> = placed(model, &frame.guess, stride);
    let split = frame.observed.count() - frame.outliers;
    let (scan, outliers) = frame.observed.points().split_at(split);
    let scan = thinned(scan);
    let outliers = thinned(outliers);
    let everything: Vec<Point> = scan.iter().chain(&guessed).copied().collect();
    let mut figure = Figure::new(View::facing(&everything, &Vector3::new(0.5, -0.6, 1.0)));
    figure
        .panel(situation.name)
        .dots(&guessed, Ink::Model)
        .dots(&scan, Ink::Observed)
        .dots(&outliers, Ink::Outlier);
    Ok(figure.render()?)
}

fn placed(model: &Model, pose: &RigidTransform, stride: usize) -> Vec<Point> {
    model
        .observed
        .cloud()
        .points()
        .iter()
        .step_by(stride)
        .map(|point| pose.apply(point))
        .collect()
}

fn thinned(points: &[Point]) -> Vec<Point> {
    let stride = points.len().div_ceil(DRAWN_SCAN_POINTS).max(1);
    points.iter().step_by(stride).copied().collect()
}
