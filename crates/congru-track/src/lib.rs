//! Follow a rigid model through a stream of point clouds. Two phases: while
//! frames are trusted, predict the pose and refine it locally; after a
//! failed frame, find the model again over all rotations.

mod filter;
mod noise;
mod sensor;
mod settings;
mod tracker;
mod view;

pub use filter::{Gains, MotionFilter};
pub use sensor::SensorView;
pub use settings::{Agility, Health, Settings};
pub use tracker::{Phase, Tracker, Update};
pub use view::{ModelView, WholeModel};
