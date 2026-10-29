use std::collections::BTreeMap;

use super::{Distance, Point, PointCloud, Vector};

impl PointCloud {
    /// One point per occupied cube of side `size`, the mean of the points in
    /// it: a density that no longer depends on the distance to the sensor,
    /// and the noise of each cube averaged down. Deterministic order.
    #[must_use]
    pub fn voxel_downsample(&self, size: Distance) -> Self {
        let mut cells: BTreeMap<[i64; 3], (Vector, usize)> = BTreeMap::new();
        for point in self.points() {
            let key = point
                .coords
                .map(|coordinate| cell_of(coordinate / size.get()));
            let cell = cells
                .entry([key.x, key.y, key.z])
                .or_insert((Vector::zeros(), 0));
            cell.0 += point.coords;
            cell.1 += 1;
        }
        let points: Vec<Point> = cells
            .into_values()
            .map(|(sum, count)| Point::from(sum / super::count_as_float(count)))
            .collect();
        Self::new(points).unwrap_or_else(|_| self.clone())
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "a finite coordinate over a positive size, floored; saturates past i64"
)]
fn cell_of(scaled: f64) -> i64 {
    scaled.floor() as i64
}
