//! The realistic scenes filmed by a camera in a hand: in each, the tracker
//! finds the target and holds on to it while the view wanders.

mod breakdown;
mod film;

use std::io::Write;
use std::path::{Path, PathBuf};

use congru_scene::Scene;

use crate::Summary;
use crate::picture::{self, Tile};
pub use breakdown::breakdown;
use film::film;

const FRAMES: usize = 72;
const WITHIN_SURFACE: f64 = 0.02;

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let frames = std::env::var("CONGRU_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(FRAMES);
    let only = std::env::var("CONGRU_SCENE").ok();
    let mut scenes = load(Path::new("scenes"))?;
    scenes.retain(|scene| only.as_ref().is_none_or(|name| *name == scene.name));
    let runs: Vec<(String, Vec<Tile>)> = scenes
        .iter()
        .map(|scene| {
            let started = std::time::Instant::now();
            let (tiles, surface): (Vec<Tile>, Vec<f64>) = film(scene, frames)?.into_iter().unzip();
            report(&scene.name, started.elapsed().as_secs_f64(), surface);
            Ok((scene.name.clone(), tiles))
        })
        .collect::<anyhow::Result<_>>()?;
    let mut grid: Vec<Vec<Tile>> = (0..frames).map(|_| Vec::new()).collect();
    for (_, tiles) in &runs {
        for (frame, tile) in grid.iter_mut().zip(tiles) {
            frame.push(tile.clone());
        }
    }
    picture::write(target, &grid, 3)?;
    Ok(runs
        .iter()
        .map(|(name, tiles)| {
            Summary::of(
                name.clone().leak(),
                tiles.iter().map(|tile| tile.error_degrees),
            )
        })
        .collect())
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

/// The surface error of one scene, on the side of the tables: the progress of a
/// long run.
fn report(name: &str, seconds: f64, mut surface: Vec<f64>) {
    let frames = surface.len();
    let within = surface.iter().filter(|off| **off < WITHIN_SURFACE).count();
    surface.sort_by(f64::total_cmp);
    let _ = writeln!(
        std::io::stderr().lock(),
        "{name}: {seconds:.1} s; surface off by a median {:.2} % of its size, {within} of {frames} within {:.0} %",
        100.0 * surface[frames / 2],
        100.0 * WITHIN_SURFACE,
    );
}
