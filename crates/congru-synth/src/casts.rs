#[expect(
    clippy::cast_precision_loss,
    reason = "point counts stay far below 2^52"
)]
pub(crate) const fn as_float(count: usize) -> f64 {
    count as f64
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a count already rounded and non-negative, saturating past the type"
)]
pub(crate) fn as_count(value: f64) -> usize {
    value.round().max(0.0) as usize
}
