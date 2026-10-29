//! Figures of the documentation, computed from real alignments.

mod draw;
mod figures;

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let folder = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("docs/figures"), PathBuf::from);
    figures::write_all(&folder)
}
