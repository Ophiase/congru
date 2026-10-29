//! Registration problems with a known answer.
//!
//! A model is corrupted the way a sensor would see it — each point resampled
//! with Gaussian noise, outliers scattered around, part of the view cropped
//! away — then moved by a random rigid motion. The motion is kept, so any
//! aligner can be scored against the truth.

mod casts;
mod corruption;
mod motion;
mod scenario;
pub mod sequence;
pub mod shapes;

pub use corruption::{Corruption, Outliers, Resampling};
pub use motion::MotionBounds;
pub use scenario::Scenario;
