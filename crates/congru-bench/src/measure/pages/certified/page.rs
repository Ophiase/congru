use std::fmt::Write;

use congru_vision::pixel::coordinate;

use super::{Graded, Row};

/// The proof stops at 1 mm, so no gap can be shown under that.
const WITHIN_MILLIMETRES: f64 = 2.0;
const WITHIN_DEGREES: f64 = 5.0;
const INTRO: &str = "# Graded against a proof\n\n\
Written by `just bench-certified`, with [this bound](certified-method.md). Few\n\
marks on a part, their correspondences unknown, some of them strays.\n\n\
- **Gap**: how far a method's value can be above the optimum, as an rms in mm, on\n  \
the instances the proof closed: elsewhere it is only a ceiling. The proof stops\n  \
at 1 mm, so every gap counts that 1 mm.\n\
- **Near the truth**: its pose within 5° of the motion that made the marks. A gear\n  \
turned by some teeth counts as far, though it is a right answer.\n\n";

/// The page: how each method did, then what the proof itself says.
pub(super) fn written(rows: &[Row]) -> String {
    let mut page = String::from(INTRO);
    let closed = rows.iter().filter(|row| row.closed).count();
    let total = rows.len();
    page +=
        "## Methods\n\n| Method | Mean gap, mm | Worst, mm | Within 2 mm | Near the truth, all |\n";
    page += "| --- | ---: | ---: | ---: | ---: |\n";
    let names: Vec<&str> = rows
        .first()
        .map(|row| row.graded.iter().map(|one| one.method).collect())
        .unwrap_or_default();
    for name in names {
        let of = |closed_only: bool| -> Vec<&Graded> {
            rows.iter()
                .filter(|row| row.closed || !closed_only)
                .flat_map(|row| &row.graded)
                .filter(|one| one.method == name)
                .collect()
        };
        page += &method_line(name, &of(true), &of(false));
    }
    let optimum_near = rows
        .iter()
        .filter(|row| row.optimum_turned < WITHIN_DEGREES)
        .count();
    let chosen = rows
        .iter()
        .filter(|row| row.scout_closed && row.spread < 2.0 * WITHIN_DEGREES)
        .count();
    let _ = write!(
        page,
        "\n## What the proof says\n\n\
- **Closed**: {closed} of {total} within the budget; elsewhere the bound holds, far.\n\
- **The best pose is near the truth**: {optimum_near} of {total}.\n\
- **The data choose**: on {chosen} of {total}, every pose not excluded is within\n  \
10°; a `+` below says more may be left.\n\n"
    );
    page + &instance_table(rows)
}

fn instance_table(rows: &[Row]) -> String {
    let mut table = String::from(
        "## Instances\n\n| Instance | Closed | Best pose off, ° | Not excluded, ° | Seconds |\n\
| --- | --- | ---: | ---: | ---: |\n",
    );
    for row in rows {
        let _ = writeln!(
            table,
            "| {} | {} | {:.1} | {:.0}{} | {:.0} |",
            row.name,
            if row.closed { "yes" } else { "no" },
            row.optimum_turned,
            row.spread,
            if row.scout_closed { "" } else { "+" },
            row.seconds,
        );
    }
    table
}

fn method_line(name: &str, closed: &[&Graded], all: &[&Graded]) -> String {
    let count = coordinate(closed.len().max(1));
    let gaps: Vec<f64> = closed.iter().map(|one| 1e3 * one.gap.max(0.0)).collect();
    let mean = gaps.iter().sum::<f64>() / count;
    let worst = gaps.iter().copied().fold(0.0, f64::max);
    let within = gaps
        .iter()
        .filter(|gap| **gap <= WITHIN_MILLIMETRES)
        .count();
    let near = all.iter().filter(|one| one.turned < WITHIN_DEGREES).count();
    let (of_closed, of_all) = (closed.len(), all.len());
    format!("| {name} | {mean:.2} | {worst:.2} | {within} of {of_closed} | {near} of {of_all} |\n")
}
