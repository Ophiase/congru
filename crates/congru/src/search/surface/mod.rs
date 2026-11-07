mod field;
mod filling;
#[cfg(test)]
mod tests;
mod tree;
mod triangle;

use nalgebra::Unit;

pub use field::DistanceField;
pub use filling::Grid;
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

    /// [`Surface::closest`] for each query, in order: on every core by default, on
    /// a device where a shape has one.
    fn closest_all(&self, queries: &[Point]) -> Vec<Option<Closest>> {
        crate::search::map(queries, |query| self.closest(query))
    }
}
