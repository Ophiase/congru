use std::fmt::Write;

use super::Row;
use super::tables::distinct;
use crate::catalog::{METHODS, Model};

/// What was compared, and on what.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn catalogue(models: &[Model]) -> Result<String, std::fmt::Error> {
    let mut text = String::new();
    writeln!(text, "## Methods and models\n")?;
    writeln!(text, "| Method | Suite | What it is |\n| --- | --- | --- |")?;
    for method in &METHODS {
        writeln!(
            text,
            "| {} | {:?} | {} |",
            method.name, method.family, method.summary
        )?;
    }
    writeln!(
        text,
        "\n| Model | What it tests | Size |\n| --- | --- | --- |"
    )?;
    for model in models {
        writeln!(
            text,
            "| {} | {} | {:.3} |",
            model.name, model.summary, model.span
        )?;
    }
    writeln!(text)?;
    Ok(text)
}

/// For each model and class of situation, the method with the highest
/// success, ties broken by time.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn recommendations(rows: &[Row]) -> Result<String, std::fmt::Error> {
    let mut text = String::new();
    writeln!(text, "## Recommendations\n")?;
    writeln!(
        text,
        "The best one-shot method for each class of situation, by mean success,\nties broken by time.\n"
    )?;
    let models = distinct(rows.iter().map(|row| row.model));
    writeln!(
        text,
        "| Class | {} |\n| --- |{}",
        models.join(" | "),
        " --- |".repeat(models.len())
    )?;
    for class in distinct(rows.iter().map(|row| row.class)) {
        let picks: Vec<String> = models
            .iter()
            .map(|model| best(rows, model, class))
            .collect();
        writeln!(text, "| {class} | {} |", picks.join(" | "))?;
    }
    writeln!(text)?;
    Ok(text)
}

#[expect(clippy::cast_precision_loss, reason = "a handful of situations")]
fn best(rows: &[Row], model: &str, class: &str) -> String {
    let mine: Vec<&Row> = rows
        .iter()
        .filter(|row| row.model == model && row.class == class)
        .collect();
    let score = |method: &str| {
        let theirs: Vec<&&Row> = mine.iter().filter(|row| row.method == method).collect();
        let count = theirs.len().max(1) as f64;
        let success = theirs.iter().map(|row| row.summary.success).sum::<f64>() / count;
        let time = theirs
            .iter()
            .map(|row| row.summary.median_milliseconds)
            .sum::<f64>()
            / count;
        (success, time)
    };
    distinct(mine.iter().map(|row| row.method))
        .into_iter()
        .map(|method| (method, score(method)))
        .max_by(|(_, left), (_, right)| left.0.total_cmp(&right.0).then(right.1.total_cmp(&left.1)))
        .map_or_else(String::new, |(method, (success, _))| {
            format!("{method} ({:.0} %)", 100.0 * success)
        })
}
