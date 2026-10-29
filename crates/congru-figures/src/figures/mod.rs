mod marks;
mod minimum;
mod outliers;
mod problem;
mod steps;
mod toy;

use std::path::Path;

use anyhow::Context;

pub fn write_all(folder: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(folder)?;
    for (name, svg) in [
        ("problem", problem::figure().context("problem")?),
        ("icp-steps", steps::figure().context("icp-steps")?),
        ("local-minimum", minimum::figure().context("local-minimum")?),
        ("outliers", outliers::figure().context("outliers")?),
        ("marks-on-mesh", marks::figure().context("marks-on-mesh")?),
    ] {
        std::fs::write(folder.join(format!("{name}.svg")), svg)?;
    }
    Ok(())
}
