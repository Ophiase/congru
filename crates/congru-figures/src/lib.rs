//! Figures drawn from real runs as SVG: panels side by side, one camera,
//! colours that follow the reader's light or dark theme.

pub mod draw;
mod sheet;

pub use draw::{Figure, Ink, View};
pub use sheet::sheet;
