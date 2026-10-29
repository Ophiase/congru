use std::fmt::Write;

use super::Row;

/// One table per model: a row per situation, a column per method.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn registration_tables(rows: &[Row]) -> Result<String, std::fmt::Error> {
    grid("Registration, one frame", rows)
}

/// One table per model: a row per sequence, a column per method; a trial is
/// a frame.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn tracking_tables(rows: &[Row]) -> Result<String, std::fmt::Error> {
    grid("Tracking, frame after frame", rows)
}

fn grid(title: &str, rows: &[Row]) -> Result<String, std::fmt::Error> {
    let mut text = String::new();
    writeln!(text, "## {title}\n")?;
    for model in distinct(rows.iter().map(|row| row.model)) {
        let methods = distinct(
            rows.iter()
                .filter(|row| row.model == model)
                .map(|row| row.method),
        );
        writeln!(text, "### {model}\n")?;
        writeln!(text, "| Situation | {} |", methods.join(" | "))?;
        writeln!(text, "| --- |{}", " --- |".repeat(methods.len()))?;
        for situation in distinct(
            rows.iter()
                .filter(|row| row.model == model)
                .map(|row| row.situation),
        ) {
            let cells: Vec<String> = methods
                .iter()
                .map(|method| {
                    rows.iter()
                        .find(|row| {
                            row.model == model
                                && row.situation == situation
                                && row.method == *method
                        })
                        .map_or_else(String::new, cell)
                })
                .collect();
            writeln!(text, "| {situation} | {} |", cells.join(" | "))?;
        }
        writeln!(text)?;
    }
    Ok(text)
}

fn cell(row: &Row) -> String {
    let degrees = row.summary.median_degrees;
    let error = if degrees.is_finite() {
        format!("{degrees:.2}°")
    } else {
        "–".to_owned()
    };
    format!("{:.0} % · {error}", 100.0 * row.summary.success)
}

pub(super) fn distinct<'item>(items: impl Iterator<Item = &'item str>) -> Vec<&'item str> {
    let mut seen: Vec<&str> = Vec::new();
    for item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}
