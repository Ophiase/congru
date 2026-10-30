//! The demos of milestone 1, written as animations with their measures:
//! `clouds` follows a model through noisy point clouds, `camera` through a
//! simulated depth camera.

mod camera;
mod clouds;
mod picture;
mod summary;
mod tour;

use std::io::Write;
use std::path::PathBuf;

use anyhow::bail;

pub use summary::Summary;

fn main() -> anyhow::Result<()> {
    let mut arguments = std::env::args_os().skip(1);
    let demo = arguments.next().and_then(|demo| demo.into_string().ok());
    let target = arguments.next().map(PathBuf::from);
    let summaries = match demo.as_deref() {
        Some("clouds") => {
            clouds::run(&target.unwrap_or_else(|| PathBuf::from("docs/figures/clouds.gif")))?
        }
        Some("camera") => {
            camera::run(&target.unwrap_or_else(|| PathBuf::from("docs/figures/tracking.gif")))?
        }
        Some("breakdown") => {
            let scenes = tour::load(std::path::Path::new("scenes"))?;
            let scene = scenes
                .iter()
                .find(|scene| {
                    scene.name
                        == std::env::var("CONGRU_SCENE")
                            .unwrap_or_else(|_| "bin of parts".to_owned())
                })
                .ok_or_else(|| anyhow::anyhow!("no such scene"))?;
            tour::breakdown(
                scene,
                &target.unwrap_or_else(|| PathBuf::from("docs/figures/breakdown.gif")),
            )?;
            Vec::new()
        }
        Some("tour") => {
            tour::run(&target.unwrap_or_else(|| PathBuf::from("docs/figures/scenes.gif")))?
        }
        _ => bail!("usage: congru-demo clouds|camera|tour [animation.gif]"),
    };
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "| Run | Median error | Frames within 2° |\n| --- | --- | --- |"
    )?;
    for summary in summaries {
        writeln!(output, "{summary}")?;
    }
    Ok(())
}
