use std::fmt;

const WITHIN_DEGREES: f64 = 2.0;

/// How a run went, over all its frames.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub name: &'static str,
    pub median_degrees: f64,
    pub within: usize,
    pub frames: usize,
}

impl Summary {
    pub fn of(name: &'static str, errors: impl Iterator<Item = f64>) -> Self {
        let mut errors: Vec<f64> = errors.collect();
        errors.sort_by(f64::total_cmp);
        Self {
            name,
            median_degrees: errors.get(errors.len() / 2).copied().unwrap_or(f64::NAN),
            within: errors
                .iter()
                .filter(|error| **error < WITHIN_DEGREES)
                .count(),
            frames: errors.len(),
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "| {} | {:.2}° | {} / {} |",
            self.name, self.median_degrees, self.within, self.frames
        )
    }
}
