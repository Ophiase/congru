//! Follow a rigid model through a stream of point clouds. Two phases: while
//! frames are trusted, predict the pose and refine it locally; after a
//! failed frame, find the model again over all rotations.

mod filter;
mod follower;
mod noise;
mod settings;
mod tracker;
mod view;

pub use filter::{Gains, MotionFilter, MotionModel};
pub use follower::{Answer, Follower, Hold, Sight, crowding};
pub use settings::{Agility, Health, Settings};
pub use tracker::{Phase, Tracker, Update};
pub use view::{CameraView, ModelView, SensorView, WholeModel};
