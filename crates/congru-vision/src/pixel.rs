//! Between pixel indices and continuous image coordinates: pixel `i`
//! covers `[i, i + 1)`, its centre at `i + 0.5`.

#[expect(
    clippy::cast_precision_loss,
    reason = "pixel indices stay far below 2^52"
)]
#[must_use]
pub const fn coordinate(index: usize) -> f64 {
    index as f64
}

/// The pixel covering a continuous coordinate, or `None` outside `0..size`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "floored and checked to lie in 0..size first"
)]
#[must_use]
pub fn containing(coordinate_value: f64, size: usize) -> Option<usize> {
    let floored = coordinate_value.floor();
    (floored >= 0.0 && floored < coordinate(size)).then_some(floored as usize)
}
