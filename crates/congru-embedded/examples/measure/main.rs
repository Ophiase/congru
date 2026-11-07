//! Writes `docs/benchmark/embedded.md`: the code size of the bare-metal images,
//! the memory of the data, and time and accuracy against the main crate.
//!
//! `measure <folder of the probe images> <page>`

mod aligners;
mod elf;
mod page;
mod plan;
mod trials;

use std::path::{Path, PathBuf};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let arguments: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [images, destination] = arguments.as_slice() else {
        anyhow::bail!("usage: measure <folder of the probe images> <page>");
    };
    let single = image_of(images, "probe-f32")?;
    let double = image_of(images, "probe-f64")?;
    let scenes = trials::run()?;
    std::fs::write(destination, page::render(&single, &double, &scenes))?;
    Ok(())
}

fn image_of(folder: &Path, name: &str) -> anyhow::Result<elf::Image> {
    let path = folder.join(name);
    let bytes = std::fs::read(&path).with_context(|| path.display().to_string())?;
    elf::Image::read(&bytes)
}
