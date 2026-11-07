use std::fmt::Write;
use std::path::Path;

use super::film::film;
use super::{FRAMES, Measures, load, wrapped};
use crate::picture::{self, Tile};

/// A scene holds at this many frames of [`FRAMES`] within the bound.
const HOLDS: usize = 60;
const INTRO: &str = "# The failing ones\n\n\
Every scene under each box, kept here while it fails: written by `just failing`,\n\
which empties this page as they are fixed. All simulated.\n\n\
- **A failing scene**: under 60 of 72 frames within 2 % of the object's size.\n\
- **The box** stands for a detector: its shift and its growth are shares of the\n  \
object's size. [Why it matters](../ideas/method/box-as-test.md).\n\n";

struct Case {
    name: &'static str,
    folder: &'static str,
    shift: f64,
    growth: f64,
}

const CASES: [Case; 3] = [
    Case {
        name: "the usual box, ±5 %, +20 %",
        folder: "tight",
        shift: 0.05,
        growth: 0.2,
    },
    Case {
        name: "a realistic box, ±10 %, +40 %",
        folder: "realistic",
        shift: 0.1,
        growth: 0.4,
    },
    Case {
        name: "a wide box, ±25 %, +100 %",
        folder: "wide",
        shift: 0.25,
        growth: 1.0,
    },
];

/// Films every scene under every box, and writes `docs/failing.md` with those that
/// fail, one line per scene that holds.
///
/// # Errors
/// A scene that cannot be loaded or filmed, or a file that cannot be written.
pub fn failing(docs: &Path) -> anyhow::Result<()> {
    let scenes = load(Path::new("scenes"))?;
    let folder = docs.join("figures/failing");
    let mut page = String::from(INTRO);
    for case in &CASES {
        let mut failed = String::new();
        let mut holding = Vec::new();
        for scene in &scenes {
            let shots = film(scene, FRAMES, (case.shift, case.growth))?;
            let measures = Measures::of(&shots);
            if measures.within_surface >= HOLDS {
                holding.push(scene.name.as_str());
                continue;
            }
            let name = format!("{}-{}", case.folder, scene.name.replace(' ', "-"));
            let drawn: Vec<Vec<Tile>> = shots.iter().map(|shot| shot.tiles.clone()).collect();
            std::fs::create_dir_all(&folder)?;
            picture::write(&folder.join(format!("{name}.gif")), &drawn, 3)?;
            writeln!(
                failed,
                "### {}\n\n![{name}](figures/failing/{name}.gif)\n\n{}\n",
                scene.name,
                wrapped(&format!("{}.", measures.line())),
            )?;
        }
        writeln!(
            page,
            "## Under {}\n\n{}\n\n{}\n",
            case.name,
            wrapped(&format!("Holding: {}.", holding.join(", "))),
            failed.trim_end()
        )?;
    }
    std::fs::write(docs.join("failing.md"), page.trim_end().to_owned() + "\n")?;
    Ok(())
}
