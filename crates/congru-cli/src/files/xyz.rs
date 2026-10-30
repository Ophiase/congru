use anyhow::{Context, bail};

use super::Raw;

/// One point per line, its first three columns; empty lines and lines
/// starting with `#` are skipped.
pub fn parse(bytes: &[u8]) -> anyhow::Result<Raw> {
    let text = std::str::from_utf8(bytes).context("not UTF-8 text")?;
    let mut coordinates = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split_whitespace().take(3).collect();
        if columns.len() < 3 {
            bail!("line {}: expected x y z", index + 1);
        }
        for column in columns {
            coordinates.push(
                column
                    .parse::<f64>()
                    .with_context(|| format!("line {}: {column} is not a number", index + 1))?,
            );
        }
    }
    Ok(Raw {
        coordinates,
        triangles: Vec::new(),
    })
}
