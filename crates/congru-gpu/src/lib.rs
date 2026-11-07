//! A [`DistanceField`] read on a GPU: the field in a buffer, one compute shader, one
//! read per point. Optional: no device, no [`GpuField`], and the field reads itself.

mod device;

use congru::{Closest, DistanceField, Point, Surface};
use device::Reader;
use nalgebra::Unit;

/// The batch sizes tried when a field meets its device, largest last.
const TRIED: [usize; 2] = [100_000, 1_000_000];

/// A field with a GPU to read it, which it asks only from the batch size where the
/// GPU was measured faster than the CPU, on this machine: an old card may never be.
#[derive(Debug)]
pub struct GpuField {
    field: DistanceField,
    reader: Reader,
    fewest_on_device: usize,
}

impl GpuField {
    /// Gives back `field` when no GPU answers: read it directly then.
    ///
    /// # Errors
    /// The field itself, without a device.
    pub fn new(field: DistanceField) -> Result<Self, DistanceField> {
        let Some(reader) = Reader::new(&field) else {
            return Err(field);
        };
        let mut built = Self {
            field,
            reader,
            fewest_on_device: usize::MAX,
        };
        built.fewest_on_device = built.crossover();
        Ok(built)
    }

    /// The smallest batch tried that the device reads faster than the CPU.
    fn crossover(&self) -> usize {
        let grid = self.field.grid();
        let middle = grid.origin
            + congru::Vector::from_fn(|axis, _| {
                grid.voxel * f64::from(u32::try_from(grid.sizes[axis] / 2).unwrap_or(0))
            });
        TRIED
            .into_iter()
            .find(|size| {
                let queries = vec![Point::from(middle); *size];
                let device = timed(|| {
                    self.closest_on_device(&queries);
                });
                let cpu = timed(|| {
                    self.field.closest_all(&queries);
                });
                device < cpu
            })
            .unwrap_or(usize::MAX)
    }

    /// From how many points a batch goes to the device; `usize::MAX` says never.
    #[must_use]
    pub const fn fewest_on_device(&self) -> usize {
        self.fewest_on_device
    }

    /// The device's name, for a report.
    #[must_use]
    pub fn device(&self) -> &str {
        self.reader.name()
    }

    /// The nearest point for every query, on the device however few they are.
    pub fn closest_on_device(&self, queries: &[Point]) -> Vec<Option<Closest>> {
        let grid = self.field.grid();
        self.reader
            .read(queries)
            .into_iter()
            .zip(queries)
            .map(|([along_x, along_y, along_z, value], query)| {
                let value = f64::from(value);
                if value.abs() >= grid.band - grid.voxel {
                    return None;
                }
                let gradient =
                    congru::Vector::new(f64::from(along_x), f64::from(along_y), f64::from(along_z));
                let normal = Unit::try_new(gradient, f64::MIN_POSITIVE.sqrt())?;
                Some(Closest {
                    point: query - normal.into_inner() * value,
                    normal,
                    squared_distance: value * value,
                })
            })
            .collect()
    }
}

impl Surface for GpuField {
    fn closest(&self, query: &Point) -> Option<Closest> {
        self.field.closest(query)
    }

    fn closest_all(&self, queries: &[Point]) -> Vec<Option<Closest>> {
        if queries.len() < self.fewest_on_device {
            self.field.closest_all(queries)
        } else {
            self.closest_on_device(queries)
        }
    }
}

fn timed(work: impl FnOnce()) -> std::time::Duration {
    let started = std::time::Instant::now();
    work();
    started.elapsed()
}

#[cfg(test)]
mod tests {
    use congru::{Distance, PointCloud, TriangleMesh};

    use super::*;

    fn cube() -> Result<TriangleMesh, congru::Error> {
        let corners: Vec<Point> = (0..8)
            .map(|bits| {
                Point::new(
                    0.2 * f64::from(bits & 1),
                    0.2 * f64::from(bits >> 1 & 1),
                    0.2 * f64::from(bits >> 2 & 1),
                )
            })
            .collect();
        let triangles = [
            [0, 2, 1],
            [1, 2, 3],
            [4, 5, 6],
            [5, 7, 6],
            [0, 1, 4],
            [1, 5, 4],
            [2, 6, 3],
            [3, 6, 7],
            [0, 4, 2],
            [2, 4, 6],
            [1, 3, 5],
            [3, 7, 5],
        ];
        TriangleMesh::new(PointCloud::new(corners)?, triangles.to_vec())
    }

    #[test]
    fn the_device_reads_what_the_cpu_reads() -> Result<(), congru::Error> {
        let field =
            DistanceField::from_mesh(&cube()?, Distance::new(0.002)?, Distance::new(0.03)?)?;
        let queries: Vec<Point> = (0..500)
            .map(|step| {
                let along = 0.0004 * f64::from(step);
                Point::new(along, 0.21 - along * 0.05, 0.1 + along * 0.3)
            })
            .collect();
        let expected = field.closest_all(&queries);
        let Ok(gpu) = GpuField::new(field) else {
            return Ok(());
        };
        let found = gpu.closest_on_device(&queries);
        for (found, expected) in found.iter().zip(&expected) {
            match (found, expected) {
                (Some(found), Some(expected)) => {
                    assert!((found.point - expected.point).norm() < 1e-4);
                }
                (None, None) => {}
                _ => assert_eq!(found.is_some(), expected.is_some()),
            }
        }
        Ok(())
    }
}
