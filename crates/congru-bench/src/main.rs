//! The benchmark of congru: every method of the catalogue, on every model,
//! in every situation of the suites, over many seeds; written as the tables
//! of `docs/benchmark/results.md`.
//!
//! `congru-bench [--tuning] [--seeds N] [--fresh] [--realistic] [results.md]`, or
//! `congru-bench --scenes` to draw every situation into `docs/benchmark/`: the tuning
//! seeds are for trying ideas, the evaluation seeds for the published
//! tables. Every finished cell is kept in `target/bench/`, and the page is
//! rewritten as cells finish: an interrupted run resumes where it stopped,
//! `--fresh` starts over after a change of the code.

mod catalog;
mod measure;
mod report;
mod session;
mod store;
mod suite;

use std::path::PathBuf;

use session::Session;
use store::Store;
use suite::Seeds;

const DEFAULT_SEEDS: u64 = 16;

fn main() -> anyhow::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| arguments.iter().any(|argument| argument == name);
    let pool = if flag("--tuning") {
        Seeds::Tuning
    } else {
        Seeds::Evaluation
    };
    let seeds = arguments
        .windows(2)
        .find(|pair| pair[0] == "--seeds")
        .and_then(|pair| pair[1].parse().ok())
        .unwrap_or(DEFAULT_SEEDS);
    let page = arguments
        .iter()
        .rfind(|argument| {
            std::path::Path::new(argument)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .map_or_else(|| PathBuf::from("docs/benchmark/results.md"), PathBuf::from);
    if flag("--scenes") {
        return draw_scenes();
    }
    if flag("--breaking") {
        return measure::breaking::run(std::path::Path::new("docs/benchmark/breaking.md"));
    }
    if flag("--certified") {
        return measure::certified::run(std::path::Path::new("docs/benchmark/certified.md"));
    }
    let cache = PathBuf::from(format!("target/bench/{pool:?}-{seeds}.tsv").to_lowercase());
    let mut session = Session::new(
        catalog::models()?,
        pool,
        seeds,
        Store::open(cache, flag("--fresh"))?,
        page,
    );
    session.run(flag("--realistic"))
}

fn draw_scenes() -> anyhow::Result<()> {
    let models = catalog::models()?;
    let folder = std::path::Path::new("docs/benchmark/scenes");
    let simple = report::scenes(&models, folder)?;
    let scenes = report::load_scenes(std::path::Path::new("scenes"))?;
    let realistic = report::realistic(&scenes, &folder.join("realistic.png"))?;
    std::fs::write("docs/benchmark/scenes.md", report::scenes_page(&realistic))?;
    std::fs::write("docs/benchmark/scenes-simple.md", simple)?;
    Ok(())
}
