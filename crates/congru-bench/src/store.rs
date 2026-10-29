use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use crate::measure::Summary;

/// The summaries already computed, one line each in a tab-separated file:
/// an interrupted run keeps what it finished, and the next one resumes.
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    known: HashMap<String, Summary>,
}

impl Store {
    /// Reads `path` if it exists; `fresh` forgets it first.
    pub fn open(path: PathBuf, fresh: bool) -> std::io::Result<Self> {
        if fresh && path.exists() {
            std::fs::remove_file(&path)?;
        }
        let known = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter_map(parse)
            .collect();
        Ok(Self { path, known })
    }

    pub fn get(&self, key: &str) -> Option<Summary> {
        self.known.get(key).copied()
    }

    pub fn put(&mut self, key: String, summary: Summary) -> std::io::Result<()> {
        if let Some(folder) = self.path.parent() {
            std::fs::create_dir_all(folder)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(
            file,
            "{key}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            summary.trials,
            summary.success,
            summary.success_low,
            summary.success_high,
            summary.confidently_wrong,
            summary.median_degrees,
            summary.worst_tenth_degrees,
            summary.median_milliseconds
        )?;
        self.known.insert(key, summary);
        Ok(())
    }
}

fn parse(line: &str) -> Option<(String, Summary)> {
    let mut fields = line.split('\t');
    let key = fields.next()?.to_owned();
    let numbers: Vec<f64> = fields.map(str::parse).collect::<Result<_, _>>().ok()?;
    let [
        trials,
        success,
        success_low,
        success_high,
        confidently_wrong,
        median_degrees,
        worst_tenth_degrees,
        median_milliseconds,
    ] = numbers[..]
    else {
        return None;
    };
    Some((
        key,
        Summary {
            trials: crate::measure::share_of(1, trials),
            success,
            success_low,
            success_high,
            confidently_wrong,
            median_degrees,
            worst_tenth_degrees,
            median_milliseconds,
        },
    ))
}
