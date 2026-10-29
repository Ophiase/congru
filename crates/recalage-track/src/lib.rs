//! Follow a rigid model through a stream of point clouds. Two phases: while
//! frames are trusted, predict the pose and refine it locally; after a
//! failed frame, find the model again over all rotations.

mod filter;
mod health;
mod tracker;
mod view;

pub use filter::{Gains, MotionFilter};
pub use health::Health;
pub use tracker::{Phase, Tracker, Update};
pub use view::{ModelView, WholeModel};
