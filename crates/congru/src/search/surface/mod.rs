mod field;
mod filling;
#[cfg(test)]
mod tests;
mod tree;
mod triangle;

use nalgebra::Unit;

pub use field::DistanceField;
pub use tree::MeshIndex;

use crate::{Point, Vector};

/// The nearest point of a surface, and which way it faces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Closest {
    pub point: Point,
    pub normal: Unit<Vector>,
    pub squared_distance: f64,
}

/// A shape that says how far a point is, and in which direction, however it holds
/// that: see [`MeshIndex`] and [`DistanceField`].
pub trait Surface: Sync {
    /// `None` where the shape says nothing, past a field's band.
    fn closest(&self, query: &Point) -> Option<Closest>;
}
