//! The realistic scenes filmed by a camera in a hand: in each, the tracker
//! finds the target and holds on to it while the view wanders.

mod breakdown;
mod failing;
mod film;
mod panels;
mod score;
mod zoom;

use std::io::Write;
use std::path::{Path, PathBuf};

use congru_scene::Scene;

use crate::Summary;
use crate::camera::detect::box_noise;
use crate::picture::{self, Tile};
pub use breakdown::breakdown;
pub use failing::failing;
use film::film;

const FRAMES: usize = 72;
const WITHIN_SURFACE: f64 = 0.02;
const WRONG_SURFACE: f64 = 0.1;
const WITHIN_DEGREES: f64 = 2.0;
const LINE_LENGTH: usize = 80;
const INTRO: &str = "# Scene films\n\n\
Every scene, as demo B draws it. Written by `just demo`, from real runs.\n\n\
- **Left**: the scene. **Middle**: the depth. **Right**: what the box holds, enlarged,\n  \
coloured over its own depths.\n\
- **Green outline**: the model at the pose found, the same on all three.\n\
- **The number**: how far off, as a share of the object's size; the bar turns amber\n  \
past 2 %, red past 10 %.\n\
- **Believed and wrong**: frames that gave a pose more than 10 % off, the ones that\n  \
matter most: a robot acts on them.\n\
- **The box comes from the truth**, a little larger, a little off, until a real\n  \
detector runs. Nothing else does.\n  \
[A wider box breaks some scenes](../ideas/method/box-as-test.md).\n\n";

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let frames = std::env::var("CONGRU_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(FRAMES);
    let only = std::env::var("CONGRU_SCENE").ok();
    let mut scenes = load(Path::new("scenes"))?;
    scenes.retain(|scene| only.as_ref().is_none_or(|name| *name == scene.name));
    let folder = target.with_extension("");
    std::fs::create_dir_all(&folder)?;
    let sections = scenes
        .iter()
        .map(|scene| {
            let started = std::time::Instant::now();
            let shots = film(scene, frames, box_noise())?;
            let name = scene.name.replace(' ', "-");
            let drawn: Vec<Vec<Tile>> = shots.iter().map(|shot| shot.tiles.clone()).collect();
            picture::write(&folder.join(format!("{name}.gif")), &drawn, 3)?;
            let measures = Measures::of(&shots);
            let mut output = std::io::stdout().lock();
            let mut times: Vec<f64> = shots.iter().map(|shot| shot.seconds).collect();
            times.sort_by(f64::total_cmp);
            writeln!(
                output,
                "{}: {:.1} s, {:.0} ms a frame; {}",
                scene.name,
                started.elapsed().as_secs_f64(),
                1e3 * times[times.len() / 2],
                measures.line(),
            )?;
            Ok(section(scene, &name, &measures))
        })
        .collect::<anyhow::Result<Vec<String>>>()?;
    if only.is_none()
        && let Some(docs) = target.parent().and_then(Path::parent)
    {
        let page = format!("{INTRO}{}\n", sections.concat().trim_end());
        std::fs::write(docs.join("scene-films.md"), page)?;
    }
    Ok(Vec::new())
}

/// How one scene went, over all its frames.
struct Measures {
    within_surface: usize,
    median_surface: f64,
    within_degrees: usize,
    /// Frames that answered, and were past [`WRONG_SURFACE`]: refusing would have
    /// been right.
    believed_wrong: usize,
    frames: usize,
}

impl Measures {
    fn of(shots: &[film::Shot]) -> Self {
        let mut surface: Vec<f64> = shots.iter().map(|shot| shot.off).collect();
        surface.sort_by(f64::total_cmp);
        Self {
            within_surface: surface.iter().filter(|off| **off < WITHIN_SURFACE).count(),
            median_surface: surface[surface.len() / 2],
            within_degrees: shots
                .iter()
                .filter(|shot| shot.degrees < WITHIN_DEGREES)
                .count(),
            believed_wrong: surface
                .iter()
                .filter(|off| off.is_finite() && **off >= WRONG_SURFACE)
                .count(),
            frames: shots.len(),
        }
    }

    fn line(&self) -> String {
        format!(
            "{} of {} within {:.0} % of its size, median {:.2} %; {} within {WITHIN_DEGREES:.0}°; \
             {} believed and wrong",
            self.within_surface,
            self.frames,
            100.0 * WITHIN_SURFACE,
            100.0 * self.median_surface,
            self.within_degrees,
            self.believed_wrong,
        )
    }
}

fn section(scene: &Scene, name: &str, measures: &Measures) -> String {
    format!(
        "## {}\n\n{}\n\n![{name}](figures/scenes/{name}.gif)\n\n{}\n\n",
        scene.name,
        wrapped(&scene.summary),
        wrapped(&format!("{}.", measures.line())),
    )
}

/// `text` cut between words, to the page's line length.
fn wrapped(text: &str) -> String {
    let mut lines: Vec<String> = vec![String::new()];
    for word in text.split_whitespace() {
        let line = lines.last_mut().map_or(0, |line| line.chars().count());
        if line > 0 && line + 1 + word.chars().count() > LINE_LENGTH {
            lines.push(String::new());
        }
        if let Some(line) = lines.last_mut() {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    lines.join("\n")
}

pub fn load(folder: &Path) -> anyhow::Result<Vec<Scene>> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .collect();
    paths.sort();
    Ok(paths
        .iter()
        .map(|path| Scene::load(path))
        .collect::<Result<_, _>>()?)
}
