//! The realistic scenes filmed by a camera in a hand: in each, the tracker
//! finds the target and holds on to it while the view wanders.

mod film;

use std::path::{Path, PathBuf};

use congru_scene::Scene;

use crate::Summary;
use crate::picture::{self, Tile};
use film::{FRAMES, film};

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let scenes = load(Path::new("scenes"))?;
    let runs: Vec<(String, Vec<Tile>)> = scenes
        .iter()
        .map(|scene| Ok((scene.name.clone(), film(scene)?)))
        .collect::<anyhow::Result<_>>()?;
    let mut frames: Vec<Vec<Tile>> = (0..FRAMES).map(|_| Vec::new()).collect();
    for (_, tiles) in &runs {
        for (frame, tile) in frames.iter_mut().zip(tiles) {
            frame.push(tile.clone());
        }
    }
    picture::write(target, &frames, 3)?;
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

fn load(folder: &Path) -> anyhow::Result<Vec<Scene>> {
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
