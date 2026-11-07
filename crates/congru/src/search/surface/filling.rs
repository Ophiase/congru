use super::{MeshIndex, Surface};
use crate::geometry::count_as_float;
use crate::search::map;
use crate::{Error, Point, TriangleMesh, Vector};

/// How many voxels a block's side holds: far from the surface a whole block takes
/// one query, so the cost of building follows the surface, not the volume.
const BLOCK: usize = 8;
const MOST_CELLS: usize = 1 << 31;

/// Where a field's samples lie: a sample's place is `origin + voxel * (i, j, k)`.
#[derive(Debug)]
pub struct Grid {
    pub origin: Vector,
    pub voxel: f64,
    /// How far from the surface its values are truthful.
    pub band: f64,
    pub sizes: [usize; 3],
}

impl Grid {
    pub(super) fn around(mesh: &TriangleMesh, voxel: f64, band: f64) -> Result<Self, Error> {
        let (low, high) = mesh.vertices().points().iter().fold(
            (
                Vector::repeat(f64::INFINITY),
                Vector::repeat(f64::NEG_INFINITY),
            ),
            |(low, high), point| (low.inf(&point.coords), high.sup(&point.coords)),
        );
        let margin = band + voxel;
        let reach = high - low + Vector::repeat(2.0 * margin);
        let sizes = [0, 1, 2].map(|axis| whole((reach[axis] / voxel).ceil()) + 2);
        if sizes.iter().product::<usize>() > MOST_CELLS {
            return Err(Error::InvalidParameter {
                name: "voxel",
                expected: "at most 2^31 cells",
                value: voxel,
            });
        }
        Ok(Self {
            origin: low - Vector::repeat(margin),
            voxel,
            band,
            sizes,
        })
    }

    fn node(&self, [column, row, layer]: [usize; 3]) -> Point {
        let place = Vector::new(
            count_as_float(column),
            count_as_float(row),
            count_as_float(layer),
        );
        Point::from(self.origin + place * self.voxel)
    }

    pub(super) fn filled(&self, index: &MeshIndex) -> Box<[f32]> {
        let [width, height, depth] = self.sizes;
        let mut blocks = Vec::new();
        for layer in (0..depth).step_by(BLOCK) {
            for row in (0..height).step_by(BLOCK) {
                for column in (0..width).step_by(BLOCK) {
                    blocks.push([column, row, layer]);
                }
            }
        }
        let filled = map(&blocks, |block| self.block(index, *block));
        let mut values = vec![0.0_f32; width * height * depth].into_boxed_slice();
        for (at, value) in filled.into_iter().flatten() {
            values[at] = value;
        }
        values
    }

    /// The values of one block, each with its place in the grid.
    fn block(&self, index: &MeshIndex, [column, row, layer]: [usize; 3]) -> Vec<(usize, f32)> {
        let [width, height, depth] = self.sizes;
        let half = BLOCK / 2;
        let centre = signed(index, &self.node([column + half, row + half, layer + half]));
        let reach = self.voxel * count_as_float(BLOCK) * 3.0_f64.sqrt() / 2.0;
        let far = centre.abs() > self.band + reach;
        let mut values = Vec::with_capacity(BLOCK * BLOCK * BLOCK);
        for deep in layer..(layer + BLOCK).min(depth) {
            for high in row..(row + BLOCK).min(height) {
                for wide in column..(column + BLOCK).min(width) {
                    let value = if far {
                        self.band * centre.signum()
                    } else {
                        signed(index, &self.node([wide, high, deep])).clamp(-self.band, self.band)
                    };
                    values.push((wide + width * (high + height * deep), narrowed(value)));
                }
            }
        }
        values
    }
}

fn signed(index: &MeshIndex, query: &Point) -> f64 {
    index.closest(query).map_or(f64::INFINITY, |closest| {
        let distance = closest.squared_distance.sqrt();
        if closest.normal.dot(&(query - closest.point)) < 0.0 {
            -distance
        } else {
            distance
        }
    })
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "a distance in single precision: far finer than a voxel"
)]
const fn narrowed(value: f64) -> f32 {
    value as f32
}

/// `value`, finite and not negative, as a count.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a floor or a ceiling, checked within the grid"
)]
pub(super) const fn whole(value: f64) -> usize {
    value as usize
}
