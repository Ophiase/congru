mod furniture;
mod ground;
mod outdoor;

use congru::{Error, TriangleMesh};
use serde::Deserialize;

pub use ground::ground;

/// What an object of a scene is. Each is built in its own frame: z up, its
/// base on `z = 0`, centred on the z axis, sized by the scene's `size`
/// (length, width, height, in metres) where its shape allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Bracket,
    Gear,
    Table,
    Shelf,
    Bin,
    Crate,
    Mug,
    Wall,
    Car,
    Tree,
    Building,
}

impl Kind {
    /// # Errors
    /// Never in practice; the type is the mesh constructor's.
    pub fn mesh(self, size: [f64; 3]) -> Result<TriangleMesh, Error> {
        match self {
            Self::Bracket => congru_sim::bracket(),
            Self::Gear => congru_sim::gear(),
            Self::Table => furniture::table(size),
            Self::Shelf => furniture::shelf(size),
            Self::Bin => furniture::bin(size),
            Self::Crate | Self::Wall => furniture::crate_box(size),
            Self::Mug => furniture::mug(size),
            Self::Car => outdoor::car(size),
            Self::Tree => outdoor::tree(size),
            Self::Building => outdoor::building(size),
        }
    }
}
