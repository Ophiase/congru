use std::fmt::Write;

use super::{Results, Row};

/// Every cell of the tables, one line each: the source of truth the
/// markdown page is laid out from.
///
/// # Errors
/// Never in practice; writing to a string cannot fail.
pub fn csv(results: &Results) -> Result<String, std::fmt::Error> {
    let mut text = String::from(
        "suite,model,method,situation,class,trials,success,success_low,success_high,confidently_wrong,median_degrees,worst_tenth_degrees,median_milliseconds\n",
    );
    for (suite, rows) in [
        ("registration", &results.registration),
        ("tracking", &results.tracking),
    ] {
        for row in rows {
            line(&mut text, suite, row)?;
        }
    }
    Ok(text)
}

fn line(text: &mut String, suite: &str, row: &Row) -> std::fmt::Result {
    let summary = &row.summary;
    writeln!(
        text,
        "{suite},{},{},\"{}\",{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.3}",
        row.model,
        row.method,
        row.situation,
        row.class,
        summary.trials,
        summary.success,
        summary.success_low,
        summary.success_high,
        summary.confidently_wrong,
        summary.median_degrees,
        summary.worst_tenth_degrees,
        summary.median_milliseconds
    )
}
