use super::Trial;

/// The 95 % two-sided normal quantile.
const NORMAL_QUANTILE: f64 = 1.96;

/// What a set of trials says about a method in a situation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub trials: usize,
    /// Share of successes, and its 95 % Wilson interval.
    pub success: f64,
    pub success_low: f64,
    pub success_high: f64,
    pub confidently_wrong: f64,
    /// Over every trial, failures included.
    pub median_degrees: f64,
    pub worst_tenth_degrees: f64,
    pub median_milliseconds: f64,
}

impl Summary {
    #[must_use]
    pub fn of(trials: &[Trial]) -> Self {
        let count = as_float(trials.len().max(1));
        let success = as_float(trials.iter().filter(|trial| trial.succeeded()).count()) / count;
        let (success_low, success_high) = wilson(success, count);
        let degrees = sorted(trials.iter().map(|trial| trial.rotation_degrees));
        let milliseconds = sorted(trials.iter().map(|trial| 1000.0 * trial.seconds));
        Self {
            trials: trials.len(),
            success,
            success_low,
            success_high,
            confidently_wrong: as_float(
                trials
                    .iter()
                    .filter(|trial| trial.confidently_wrong())
                    .count(),
            ) / count,
            median_degrees: quantile(&degrees, 0.5),
            worst_tenth_degrees: quantile(&degrees, 0.9),
            median_milliseconds: quantile(&milliseconds, 0.5),
        }
    }
}

fn wilson(share: f64, count: f64) -> (f64, f64) {
    let spread = NORMAL_QUANTILE * NORMAL_QUANTILE / count;
    let centre = (share + spread / 2.0) / (1.0 + spread);
    let half = NORMAL_QUANTILE * (share * (1.0 - share) / count + spread / (4.0 * count)).sqrt()
        / (1.0 + spread);
    ((centre - half).max(0.0), (centre + half).min(1.0))
}

fn sorted(values: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut values: Vec<f64> = values.collect();
    values.sort_by(f64::total_cmp);
    values
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a rank inside the slice, rounded down"
)]
fn quantile(sorted: &[f64], share: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((as_float(sorted.len() - 1) * share).floor() as usize).min(sorted.len() - 1)]
}

#[expect(
    clippy::cast_precision_loss,
    reason = "trial counts stay far below 2^52"
)]
fn as_float(count: usize) -> f64 {
    count as f64
}
