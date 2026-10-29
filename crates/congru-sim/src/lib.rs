//! A depth camera watching a machined part turn on a turntable: the part,
//! its path, and what a real sensor does to the depth.

mod degrade;
mod flex;
mod part;
mod path;
mod solids;

pub use degrade::Degradation;
pub use flex::flexed;
pub use part::{bracket, gear, surroundings, turntable};
pub use path::Turntable;
pub use solids::Solids;
