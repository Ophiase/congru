use std::fmt::Write;

use super::tables::distinct;
use super::{Results, Row};

/// A row per method: its mean success over the situations it was run on,
/// its worst situation, how often it claimed a wrong pose, and its time.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn leaderboard(results: &Results) -> Result<String, std::fmt::Error> {
    let mut text = String::new();
    writeln!(text, "## Leaderboard\n")?;
    writeln!(
        text,
        "Every situation and model weighs the same. *Confidently wrong* is the\n\
         share of trials where the method trusted a pose that missed the\n\
         tolerance: for a robot, the most dangerous outcome.\n"
    )?;
    writeln!(
        text,
        "| Method | Suite | Mean success | Worst situation | Confidently wrong | Median time |"
    )?;
    writeln!(text, "| --- | --- | --- | --- | --- | --- |")?;
    for (suite, rows) in [
        ("registration", &results.registration),
        ("tracking", &results.tracking),
    ] {
        for method in distinct(rows.iter().map(|row| row.method)) {
            let mine: Vec<&Row> = rows.iter().filter(|row| row.method == method).collect();
            writeln!(text, "{}", line(method, suite, &mine))?;
        }
    }
    writeln!(text)?;
    Ok(text)
}

#[expect(clippy::cast_precision_loss, reason = "a handful of situations")]
fn line(method: &str, suite: &str, rows: &[&Row]) -> String {
    let count = rows.len().max(1) as f64;
    let mean = rows.iter().map(|row| row.summary.success).sum::<f64>() / count;
    let worst = rows
        .iter()
        .min_by(|left, right| left.summary.success.total_cmp(&right.summary.success))
        .map_or_else(String::new, |row| {
            format!(
                "{:.0} % ({}, {})",
                100.0 * row.summary.success,
                row.situation,
                row.model
            )
        });
    let wrong = rows
        .iter()
        .map(|row| row.summary.confidently_wrong)
        .sum::<f64>()
        / count;
    let mut times: Vec<f64> = rows
        .iter()
        .map(|row| row.summary.median_milliseconds)
        .collect();
    times.sort_by(f64::total_cmp);
    let time = times.get(times.len() / 2).copied().unwrap_or(f64::NAN);
    format!(
        "| {method} | {suite} | {:.1} % | {worst} | {:.1} % | {time:.2} ms |",
        100.0 * mean,
        100.0 * wrong
    )
}
