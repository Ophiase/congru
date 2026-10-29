//! What a depth camera sees, and how to turn it into points to align: the
//! same functions serve a simulated camera and a real one.

mod camera;
mod image;
mod perception;
pub mod pixel;
mod render;

pub use camera::Camera;
pub use image::{DepthImage, Image};
pub use perception::{back_project, visible_from_origin, within, without_support};
pub use render::{Frame, render};
