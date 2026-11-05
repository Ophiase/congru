//! A page of still frames, one per way of seeing the part: where the search
//! puts the model against the truth, and what each ray says of both.

use std::fmt::Write as _;
use std::path::Path;

use congru::{Distance, RigidTransform};
use congru_vision::{Finder, Image, Observed};
use rayon::prelude::*;

use super::{PART_SPAN, STEP_SHARE};
use crate::camera::conditions::{CONDITIONS, Condition};
use crate::camera::overlay::rainbow;
use crate::camera::stage;
use crate::camera::stage::Stage;
use crate::picture::palette::{BAD, EMPTY, FAIR, GOOD, TEXT, Tone};
use crate::picture::{self, Tile, font, outline};

const PAGE: &str = "docs/projection-cases.md";
const FIGURES: &str = "docs/figures/projection";
const INTRO: &str = "# Projection: cases, frame by frame\n\
\n\
Still frames of demo B, from real runs, each answering\n\
[what a depth frame says](../ideas/problem/visibility.md). The truth only scores:\n\
the search never sees it.\n\
\n\
| Panel | What it shows |\n\
| --- | --- |\n\
| truth | The depth, near in red and far in blue, the true outline over it |\n\
| found | The same, with the outline of the pose the search answers |\n\
| rays, truth | The model at the truth, each pixel judged by its ray |\n\
| rays, found | The same at the pose found |\n\
\n\
Rays: green on the surface, amber hidden, red when the ray went through the model.\n\
White spots in the depth are holes. The bar below each panel is the rotation error.\n\
\n\
The reading compares scores: a wrong pose under the truth's score means the search\n\
never offered the truth; above it, the score itself fails. Written by\n\
`congru-demo cases`.\n\
\n\
";
const CASES: [(&str, usize); 16] = [
    ("sensor", 0),
    ("sensor", 51),
    ("shake", 40),
    ("lens", 27),
    ("flex", 30),
    ("all effects", 0),
    ("all effects", 27),
    ("room", 0),
    ("room", 7),
    ("room", 27),
    ("room", 40),
    ("in a room", 0),
    ("in a room", 7),
    ("in a room", 27),
    ("in a room", 40),
    ("in a room", 73),
];
const PANELS: usize = 4;
const READABLE: f64 = 1e4;
const RIGHT_DEGREES: f64 = 5.0;
const RIGHT_METRES: f64 = 0.02;
/// At this share of the step, the depth check reads close to the noise.
const TIGHT: f64 = 0.5;

/// Writes the page and its figures.
pub fn run() -> anyhow::Result<()> {
    let stage = stage()?;
    let step = PART_SPAN * STEP_SHARE;
    let finder = Finder::new(&stage.part, Distance::new(step)?)?;
    std::fs::create_dir_all(FIGURES)?;
    let sections = CASES
        .par_iter()
        .map(|(name, frame)| {
            let condition = CONDITIONS
                .iter()
                .find(|condition| condition.name == *name)
                .ok_or_else(|| anyhow::anyhow!("no condition {name}"))?;
            case(&stage, &finder, condition, *frame)
        })
        .collect::<anyhow::Result<Vec<String>>>()?;
    let page = format!("{INTRO}{}\n", sections.concat().trim_end());
    std::fs::write(PAGE, page)?;
    Ok(())
}

fn case(
    stage: &Stage,
    finder: &Finder,
    condition: &Condition,
    frame: usize,
) -> anyhow::Result<String> {
    let tolerance = PART_SPAN * STEP_SHARE * TIGHT;
    let (depth, truth) = stage.sensed(condition, frame)?;
    let found = finder.find(&depth, &stage.camera, f64::NEG_INFINITY);
    let pose = found.map_or(truth, |found| found.pose);
    let degrees = pose.rotation_error_to(&truth).to_degrees();
    let metres = pose.translation_error_to(&truth);
    let seen = Observed::new(depth.clone(), &stage.camera, tolerance);
    let score_of = |pose| READABLE * finder.score(&seen, &stage.camera, pose, tolerance);
    let (at_truth, at_found) = (score_of(&truth), score_of(&pose));
    let slug = format!("{}-{frame:02}", condition.name.replace(' ', "-"));
    let tiles = panels(stage, &depth, (&truth, &pose), tolerance).map(|picture| Tile {
        picture,
        error_degrees: degrees,
    });
    picture::write(
        Path::new(&format!("{FIGURES}/{slug}.gif")),
        &[tiles.to_vec()],
        PANELS,
    )?;
    let mut section = format!("## {}, frame {frame}\n\n", condition.name);
    writeln!(section, "![{slug}](figures/projection/{slug}.gif)\n")?;
    writeln!(
        section,
        "- **Off by**: {degrees:.1}° and {:.0} mm.",
        metres * 1e3
    )?;
    writeln!(
        section,
        "- **Score**: truth {at_truth:.1}, found {at_found:.1}."
    )?;
    writeln!(
        section,
        "- **Reading**: {}.\n",
        reading(degrees, metres, at_truth, at_found)
    )?;
    Ok(section)
}

fn panels(
    stage: &Stage,
    depth: &Image<f64>,
    (truth, found): (&RigidTransform, &RigidTransform),
    tolerance: f64,
) -> [Image<Tone>; PANELS] {
    let drawn = |pose: &RigidTransform| {
        let mut picture = rainbow(depth, depth);
        outline(&mut picture, &silhouette(stage, pose));
        picture
    };
    [
        (drawn(truth), "truth"),
        (drawn(found), "found"),
        (verdicts(stage, depth, truth, tolerance), "rays, truth"),
        (verdicts(stage, depth, found, tolerance), "rays, found"),
    ]
    .map(|(mut picture, label)| {
        font::write(&mut picture, 4, 4, label, TEXT);
        picture
    })
}

fn reading(degrees: f64, metres: f64, at_truth: f64, at_found: f64) -> &'static str {
    if degrees < RIGHT_DEGREES && metres < RIGHT_METRES {
        "right"
    } else if at_truth > at_found {
        "wrong, and the score knows: the search never offered the truth"
    } else {
        "wrong, and the score prefers it: the score itself fails"
    }
}

fn silhouette(stage: &Stage, pose: &RigidTransform) -> Image<bool> {
    stage
        .shoot(&stage.part, pose, &[])
        .depth
        .map(f64::is_finite)
}

/// Each pixel of the model at `pose`, coloured by what its ray says: green on
/// the surface, amber hidden, red when the ray went through, grey for a hole.
fn verdicts(
    stage: &Stage,
    depth: &Image<f64>,
    pose: &RigidTransform,
    tolerance: f64,
) -> Image<Tone> {
    let rendered = stage.shoot(&stage.part, pose, &[]).depth;
    let mut picture = Image::filled(depth.width(), depth.height(), EMPTY);
    for (column, row) in rendered.positions() {
        let expected = rendered.get(column, row);
        let value = depth.get(column, row);
        if expected.is_finite() && value.is_finite() {
            let tone = if value > expected + tolerance {
                BAD
            } else if value < expected - tolerance {
                FAIR
            } else {
                GOOD
            };
            picture.set(column, row, tone);
        }
    }
    picture
}
