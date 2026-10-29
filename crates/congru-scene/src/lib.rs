//! Ordinary scenes for congru: a camera, a ground of fractal relief, and
//! objects built from a small library of parametric props, one of them the
//! target to find. A scene is a short TOML file; see `scenes/` at the root
//! of the repository.

mod description;
mod error;
mod noise;
mod props;
mod scene;

pub use description::{CameraSpec, Description, GroundSpec, ObjectSpec};
pub use error::SceneError;
pub use noise::Fractal;
pub use props::{Kind, ground};
pub use scene::{Placed, Scene};
