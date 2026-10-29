//! The demos of milestone 1, written as animations with their measures:
//! `clouds` follows a model through noisy point clouds, `camera` through a
//! simulated depth camera.

mod camera;
mod clouds;
mod picture;
mod summary;

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
        _ => bail!("usage: recalage-demo clouds|camera [animation.gif]"),
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
