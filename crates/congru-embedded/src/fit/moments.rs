use super::Pair;
use crate::{Error, Real, Vector3};

/// Weighted first and second moments of a set of pairs.
pub(super) struct Moments<Float> {
    pub(super) source_centre: Vector3<Float>,
    pub(super) target_centre: Vector3<Float>,
    /// `Σ w (s − s̄)(t − t̄)ᵀ`, by rows.
    pub(super) covariance: [[Float; 3]; 3],
    /// `Σ w ‖s − s̄‖²`.
    pub(super) source_spread: Float,
}

const MINIMUM_PAIRS: usize = 3;

struct FirstPass<Float> {
    total: Float,
    source_centre: Vector3<Float>,
    target_centre: Vector3<Float>,
}

/// Two passes: the first finds rough centres, the second sums about them and
/// corrects them. A plain `f32` sum of many coordinates loses the digits a
/// centroid needs; the differences of the second pass are small and keep them.
pub(super) fn moments<Float: Real>(
    pairs: impl Iterator<Item = Pair<Float>> + Clone,
) -> Result<Moments<Float>, Error> {
    let rough = first_pass(pairs.clone())?;
    let (mut source_shift, mut target_shift) = (Vector3::ZERO, Vector3::ZERO);
    let mut covariance = [[Float::ZERO; 3]; 3];
    let mut source_spread = Float::ZERO;
    for pair in pairs {
        let from = pair.source - rough.source_centre;
        let to = pair.target - rough.target_centre;
        source_shift += from * pair.weight;
        target_shift += to * pair.weight;
        source_spread += from.squared_norm() * pair.weight;
        accumulate(&mut covariance, from.outer(to), pair.weight);
    }
    accumulate(
        &mut covariance,
        source_shift.outer(target_shift),
        -Float::ONE / rough.total,
    );
    source_spread -= source_shift.squared_norm() / rough.total;
    Ok(Moments {
        source_centre: rough.source_centre + source_shift * (Float::ONE / rough.total),
        target_centre: rough.target_centre + target_shift * (Float::ONE / rough.total),
        covariance,
        source_spread,
    })
}

fn first_pass<Float: Real>(
    pairs: impl Iterator<Item = Pair<Float>>,
) -> Result<FirstPass<Float>, Error> {
    let mut count = 0;
    let mut total = Float::ZERO;
    let (mut source_sum, mut target_sum) = (Vector3::ZERO, Vector3::ZERO);
    for pair in pairs {
        if pair.weight < Float::ZERO || !pair.weight.finite() {
            return Err(Error::NotFinite);
        }
        count += usize::from(pair.weight > Float::ZERO);
        total += pair.weight;
        source_sum += pair.source * pair.weight;
        target_sum += pair.target * pair.weight;
    }
    if count < MINIMUM_PAIRS {
        return Err(Error::TooFewPairs {
            found: count,
            needed: MINIMUM_PAIRS,
        });
    }
    Ok(FirstPass {
        total,
        source_centre: source_sum * (Float::ONE / total),
        target_centre: target_sum * (Float::ONE / total),
    })
}

fn accumulate<Float: Real>(sum: &mut [[Float; 3]; 3], term: [[Float; 3]; 3], weight: Float) {
    for (sum_row, term_row) in sum.iter_mut().zip(term) {
        for (entry, value) in sum_row.iter_mut().zip(term_row) {
            *entry += value * weight;
        }
    }
}
