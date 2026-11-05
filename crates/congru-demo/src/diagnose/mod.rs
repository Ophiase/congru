//! Measures that tell the steps of demo B apart. The truth only scores here.

mod rank;

pub use rank::run as rank;

const PART_SPAN: f64 = 0.25;
const STEP_SHARE: f64 = 1.0 / 20.0;
