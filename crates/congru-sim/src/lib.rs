//! A depth camera watching a machined part turn on a turntable: the part,
//! its path, and what a real sensor does to the depth.

mod degrade;
mod part;
mod path;
mod solids;

pub use degrade::Degradation;
pub use part::{bracket, gear, turntable};
pub use path::Turntable;
