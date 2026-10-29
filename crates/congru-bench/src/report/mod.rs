mod catalogue;
mod csv;
mod gallery;
mod leaderboard;
mod tables;

use std::fmt::Write;

pub use catalogue::{catalogue, recommendations};
pub use csv::csv;
pub use gallery::{load_scenes, realistic, scenes, scenes_page};
pub use leaderboard::leaderboard;
pub use tables::{registration_tables, tracking_tables};

use crate::measure::Summary;

/// Every summary of a run, by model, by situation or sequence, by method.
#[derive(Debug, Clone, PartialEq)]
pub struct Results {
    pub seeds: u64,
    pub registration: Vec<Row>,
    pub tracking: Vec<Row>,
}

/// One cell of the tables: a method on a model in one situation.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub model: &'static str,
    pub situation: &'static str,
    pub class: &'static str,
    pub method: &'static str,
    pub summary: Summary,
}

/// The whole results page, as markdown.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn page(
    results: &Results,
    models: &[crate::catalog::Model],
) -> Result<String, std::fmt::Error> {
    let mut page = String::new();
    writeln!(page, "# Benchmark results\n")?;
    writeln!(
        page,
        "Written by `just bench` from the evaluation seeds, {} per situation. The
\
         source of truth is [results.csv](results.csv): this page only lays it
\
         out. The protocol, the situations and the metrics are defined in\n\
         [protocol.md](protocol.md). A cell reads *success rate · median\n\
         rotation error*; times are from the machine that wrote this page.\n",
        results.seeds
    )?;
    page.push_str(&leaderboard(results)?);
    page.push_str(&recommendations(&results.registration)?);
    page.push_str(&catalogue(models)?);
    page.push_str(&registration_tables(&results.registration)?);
    page.push_str(&tracking_tables(&results.tracking)?);
    Ok(format!("{}\n", page.trim_end()))
}
