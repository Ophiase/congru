use nalgebra::Unit;

use super::filling::{Grid, whole};
use super::{Closest, MeshIndex, Surface};
use crate::geometry::count_as_float;
use crate::{Distance, Error, Point, TriangleMesh, Vector};

/// A [signed distance field] sampled on a regular grid, in a narrow band: the distance
/// and its direction for the price of a read, where a tree must search.
///
/// Its error is about the voxel, and it says nothing past the band.
///
/// [signed distance field]: https://en.wikipedia.org/wiki/Signed_distance_function
#[derive(Debug)]
pub struct DistanceField {
    grid: Grid,
    values: Box<[f32]>,
}

impl DistanceField {
    /// A field of cubes of side `voxel`, truthful within `band` of the surface. The
    /// sign comes from the normal of the nearest face, so a mesh must be oriented.
    ///
    /// # Errors
    /// As [`MeshIndex::new`]; [`Error::InvalidParameter`] for a grid past memory.
    pub fn from_mesh(mesh: &TriangleMesh, voxel: Distance, band: Distance) -> Result<Self, Error> {
        let index = MeshIndex::new(mesh)?;
        let grid = Grid::around(mesh, voxel.get(), band.get())?;
        let values = grid.filled(&index);
        Ok(Self { grid, values })
    }

    /// Where its samples lie.
    #[must_use]
    pub const fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Its samples, the first axis running fastest.
    #[must_use]
    pub const fn values(&self) -> &[f32] {
        &self.values
    }

    /// How many bytes its grid holds.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.values.len() * size_of::<f32>()
    }

    fn at(&self, column: usize, row: usize, layer: usize) -> f64 {
        let [width, height, _] = self.grid.sizes;
        f64::from(self.values[column + width * (row + height * layer)])
    }

    /// The signed distance at `query` and its gradient, by a trilinear blend; `None`
    /// outside the grid.
    #[must_use]
    pub fn read(&self, query: &Point) -> Option<(f64, Vector)> {
        let place = (query.coords - self.grid.origin) / self.grid.voxel;
        let mut cell = [0_usize; 3];
        let mut share = [0.0; 3];
        for axis in 0..3 {
            let floor = place[axis].floor();
            if floor < 0.0 || floor >= count_as_float(self.grid.sizes[axis] - 1) {
                return None;
            }
            cell[axis] = whole(floor);
            share[axis] = place[axis] - floor;
        }
        let [column, row, layer] = cell;
        let [wide, high, deep] = share;
        let corner =
            |right: usize, up: usize, back: usize| self.at(column + right, row + up, layer + back);
        let (c00, c10) = (corner(0, 0, 0), corner(1, 0, 0));
        let (c01, c11) = (corner(0, 1, 0), corner(1, 1, 0));
        let (d00, d10) = (corner(0, 0, 1), corner(1, 0, 1));
        let (d01, d11) = (corner(0, 1, 1), corner(1, 1, 1));
        let lower = |first: f64, second: f64| first + (second - first) * wide;
        let (e0, e1) = (lower(c00, c10), lower(c01, c11));
        let (f0, f1) = (lower(d00, d10), lower(d01, d11));
        let (g0, g1) = (e0 + (e1 - e0) * high, f0 + (f1 - f0) * high);
        let near = (c10 - c00) + ((c11 - c01) - (c10 - c00)) * high;
        let far = (d10 - d00) + ((d11 - d01) - (d10 - d00)) * high;
        let gradient = Vector::new(
            near + (far - near) * deep,
            (e1 - e0) + ((f1 - f0) - (e1 - e0)) * deep,
            g1 - g0,
        );
        Some((g0 + (g1 - g0) * deep, gradient / self.grid.voxel))
    }
}

impl Surface for DistanceField {
    fn closest(&self, query: &Point) -> Option<Closest> {
        let (value, gradient) = self.read(query)?;
        if value.abs() >= self.grid.band - self.grid.voxel {
            return None;
        }
        let normal = Unit::try_new(gradient, f64::MIN_POSITIVE.sqrt())?;
        Some(Closest {
            point: query - normal.into_inner() * value,
            normal,
            squared_distance: value * value,
        })
    }
}
