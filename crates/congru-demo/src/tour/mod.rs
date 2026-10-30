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

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let frames = std::env::var("CONGRU_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(FRAMES);
    let scenes = load(Path::new("scenes"))?;
    let runs: Vec<(String, Vec<Tile>)> = scenes
        .iter()
        .map(|scene| {
            let started = std::time::Instant::now();
            let tiles = film(scene, frames)?;
            let _ = writeln!(
                std::io::stderr().lock(),
                "{}: {:.1} s",
                scene.name,
                started.elapsed().as_secs_f64()
            );
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
