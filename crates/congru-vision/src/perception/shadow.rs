use crate::pixel::coordinate;
use crate::{Camera, DepthImage};

/// How much a violation costs against the same area of support.
const VIOLATION_WEIGHT: f64 = 1.0;
/// How far outside the silhouette its edge is read, in pixels.
const EDGE_GAP: usize = 2;
/// A sensor mixes the two sides of a depth edge over a pixel or two: the
/// image proves free space only when this whole neighbourhood sees it.
const RIM: usize = 2;

/// A model's rendered depth read against a depth image, as metric areas, in
/// the square of the unit of the depths. A pixel without a return counts for
/// nothing: it may be a hole.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewScore {
    /// The model stands where the image saw a surface.
    pub support: f64,
    /// The model stands where the image saw through.
    pub violation: f64,
    /// Something the image saw hides the model.
    pub occluded: f64,
    /// Pixels just outside the silhouette where the image jumps away from
    /// the model's edge: the object stands out there.
    pub standing: usize,
    /// Pixels just outside the silhouette where the surface goes on past the
    /// model's edge: nothing marks an object there.
    pub flush: usize,
}

impl ViewScore {
    /// Reads `rendered` against `depth`, both from `camera`. `tolerance` must
    /// exceed the sensor's noise.
    #[must_use]
    pub fn of(depth: &DepthImage, rendered: &DepthImage, camera: &Camera, tolerance: f64) -> Self {
        let mut score = Self::default();
        for (column, row) in rendered.positions() {
            let expected = rendered.get(column, row);
            let seen = depth.get(column, row);
            if !expected.is_finite() {
                if let Some(edge) = edge_depth(rendered, column, row) {
                    if seen.is_nan() || seen > edge + tolerance {
                        score.standing += 1;
                    } else if (seen - edge).abs() <= tolerance {
                        score.flush += 1;
                    }
                }
                continue;
            }
            if !seen.is_finite() {
                continue;
            }
            let side = expected / camera.focal;
            let area = side * side;
            if expected < seen - tolerance {
                if sees_through(depth, column, row, expected + tolerance) {
                    score.violation += area;
                }
            } else if expected > seen + tolerance {
                score.occluded += area;
            } else {
                score.support += area;
            }
        }
        score
    }

    /// The support less what the violations cost, times the share of the
    /// outline that stands out. Support alone rewards a flat face sunk into
    /// a flat surface, which the surface explains without any object; an
    /// outline that stands out is what only an object explains.
    #[must_use]
    pub fn value(&self) -> f64 {
        let outline = self.standing + self.flush;
        if outline == 0 {
            return 0.0;
        }
        let standing = coordinate(self.standing) / coordinate(outline);
        (self.support - VIOLATION_WEIGHT * self.violation) * standing
    }
}

/// Whether every return around a pixel lies beyond `limit`.
fn sees_through(depth: &DepthImage, column: usize, row: usize, limit: f64) -> bool {
    let columns = column.saturating_sub(RIM)..=(column + RIM).min(depth.width() - 1);
    columns.into_iter().all(|across| {
        let rows = row.saturating_sub(RIM)..=(row + RIM).min(depth.height() - 1);
        rows.into_iter().all(|down| {
            let seen = depth.get(across, down);
            seen.is_nan() || seen > limit
        })
    })
}

/// The model's depth at a silhouette pixel a few pixels from this one.
fn edge_depth(rendered: &DepthImage, column: usize, row: usize) -> Option<f64> {
    [
        (column.checked_sub(EDGE_GAP), Some(row)),
        (Some(column + EDGE_GAP), Some(row)),
        (Some(column), row.checked_sub(EDGE_GAP)),
        (Some(column), Some(row + EDGE_GAP)),
    ]
    .into_iter()
    .filter_map(|(across, down)| {
        let (across, down) = (across?, down?);
        (across < rendered.width() && down < rendered.height())
            .then(|| rendered.get(across, down))
            .filter(|value| value.is_finite())
    })
    .reduce(f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAMERA: Camera = Camera {
        width: 2,
        height: 1,
        focal: 1.0,
    };

    fn image(values: [f64; 2]) -> DepthImage {
        let mut image = DepthImage::filled(2, 1, f64::NAN);
        image.set(0, 0, values[0]);
        image.set(1, 0, values[1]);
        image
    }

    #[test]
    fn each_pixel_falls_in_the_region_its_depths_say() {
        let score = ViewScore::of(&image([2.0, 2.0]), &image([2.0, 1.0]), &CAMERA, 0.1);
        assert!((score.support - 4.0).abs() < 1e-12);
        assert!((score.violation - 1.0).abs() < 1e-12);
        let score = ViewScore::of(&image([1.0, 2.0]), &image([2.0, f64::NAN]), &CAMERA, 0.1);
        assert!((score.occluded - 4.0).abs() < 1e-12);
        assert!(score.support.abs() < 1e-12);
    }

    #[test]
    fn a_pixel_without_a_return_counts_for_nothing() {
        let score = ViewScore::of(&image([f64::NAN, 2.0]), &image([1.0, 2.0]), &CAMERA, 0.1);
        assert!(score.violation.abs() < 1e-12);
        assert!((score.support - 4.0).abs() < 1e-12);
    }
}
