mod observed;

use congru::Vector;
use nalgebra::{SMatrix, SVector};

use crate::{Camera, DepthImage};
pub use observed::Observed;

type Twist = SVector<f64, 6>;

/// A model's face and the image's surface agree within this cosine, 45°:
/// loose, since the image's normal is a blurred one.
const LEAST_COSINE: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// A direction weaker than this share of the strongest is free: below it lies only
/// rounding, and a flat view's mean, and any ratio of two, would read that.
const LEAST_DIRECTION: f64 = 1e-6;

/// How firmly the pixels a rendered model shares with a depth image pin its
/// pose: the eigenvalues of the point-to-plane matrix of those pixels, each
/// weighted by its metric area, rotations scaled by the size of what is seen. A flat face leaves three of them at
/// zero however large it is, since it says nothing of sliding along itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Information {
    pub eigenvalues: [f64; 6],
}

impl Information {
    /// Reads `rendered` against `seen`, both from `camera`.
    #[must_use]
    pub fn of(seen: &Observed, rendered: &DepthImage, camera: &Camera, tolerance: f64) -> Self {
        let elements = elements(seen, rendered, camera, tolerance);
        let total: f64 = elements.iter().map(|element| element.area).sum();
        if total <= 0.0 {
            return Self {
                eigenvalues: [0.0; 6],
            };
        }
        let centre = elements.iter().fold(Vector::zeros(), |sum, element| {
            sum + element.position * element.area
        }) / total;
        let spread = (elements
            .iter()
            .map(|element| element.area * (element.position - centre).norm_squared())
            .sum::<f64>()
            / total)
            .sqrt()
            .max(f64::EPSILON);
        let matrix = elements
            .iter()
            .fold(SMatrix::<f64, 6, 6>::zeros(), |sum, element| {
                let turn = (element.position - centre).cross(&element.normal) / spread;
                let twist = Twist::new(
                    turn.x,
                    turn.y,
                    turn.z,
                    element.normal.x,
                    element.normal.y,
                    element.normal.z,
                );
                sum + twist * twist.transpose() * element.area
            });
        let mut eigenvalues = [0.0; 6];
        for (slot, value) in eigenvalues
            .iter_mut()
            .zip(matrix.symmetric_eigenvalues().iter())
        {
            *slot = value.max(0.0);
        }
        eigenvalues.sort_by(f64::total_cmp);
        Self { eigenvalues }
    }

    /// The weakest direction: what the view pins least.
    #[must_use]
    pub const fn weakest(&self) -> f64 {
        self.eigenvalues[0]
    }

    /// The geometric mean of all six directions, each counted at
    /// [`LEAST_DIRECTION`] of the strongest at least.
    #[must_use]
    pub fn mean(&self) -> f64 {
        let floor = (self.eigenvalues[5] * LEAST_DIRECTION).max(f64::MIN_POSITIVE);
        let logarithms: f64 = self
            .eigenvalues
            .iter()
            .map(|value| value.max(floor).ln())
            .sum();
        (logarithms / 6.0).exp()
    }
}

#[derive(Debug, Clone, Copy)]
struct Element {
    position: Vector,
    normal: Vector,
    area: f64,
}

fn elements(
    observed: &Observed,
    rendered: &DepthImage,
    camera: &Camera,
    tolerance: f64,
) -> Vec<Element> {
    let (width, height) = (rendered.width(), rendered.height());
    let rows: Vec<usize> = (1..height.saturating_sub(1)).collect();
    congru::map(&rows, |row| {
        let mut elements = Vec::new();
        for column in 1..width.saturating_sub(1) {
            let expected = rendered.get(column, *row);
            let seen = observed.depth.get(column, *row);
            if !expected.is_finite() || !seen.is_finite() || (expected - seen).abs() > tolerance {
                continue;
            }
            let faces = normal_at(rendered, camera, column, *row, tolerance)
                .zip(observed.normal(column, *row));
            if let Some((normal, _)) =
                faces.filter(|(normal, image)| normal.dot(image).abs() >= LEAST_COSINE)
            {
                let side = expected / camera.focal;
                elements.push(Element {
                    position: camera.back_project(column, *row, expected).coords,
                    normal,
                    area: side * side,
                });
            }
        }
        elements
    })
    .concat()
}

/// The surface's normal at a pixel, from its four neighbours; `None` across
/// a depth edge, where a pixel has no surface of its own.
fn normal_at(
    rendered: &DepthImage,
    camera: &Camera,
    column: usize,
    row: usize,
    tolerance: f64,
) -> Option<Vector> {
    let centre = rendered.get(column, row);
    let point = |across: usize, down: usize| {
        let value = rendered.get(across, down);
        ((value - centre).abs() <= tolerance).then(|| camera.back_project(across, down, value))
    };
    let along = point(column + 1, row)? - point(column - 1, row)?;
    let downward = point(column, row + 1)? - point(column, row - 1)?;
    along.cross(&downward).try_normalize(f64::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_face_pins_nothing_along_itself() {
        let camera = Camera {
            width: 40,
            height: 40,
            focal: 40.0,
        };
        let flat = DepthImage::filled(40, 40, 1.0);
        let seen = Observed::new(flat.clone(), &camera, 0.01);
        let information = Information::of(&seen, &flat, &camera, 0.01);
        assert!(information.weakest() < 1e-9);
        assert!(information.eigenvalues[5] > 0.1);
    }

    #[test]
    fn half_a_flat_face_seen_is_worth_about_half_of_it() {
        let camera = Camera {
            width: 40,
            height: 40,
            focal: 40.0,
        };
        let flat = DepthImage::filled(40, 40, 1.0);
        let mut half = flat.clone();
        for (column, row) in flat.positions() {
            if column < 20 {
                half.set(column, row, f64::NAN);
            }
        }
        let whole = Information::of(
            &Observed::new(flat.clone(), &camera, 0.01),
            &flat,
            &camera,
            0.01,
        );
        let seen = Information::of(&Observed::new(half, &camera, 0.01), &flat, &camera, 0.01);
        let ratio = seen.mean() / whole.mean();
        assert!((0.3..0.7).contains(&ratio), "{ratio}");
    }

    #[test]
    fn a_face_across_the_seen_surface_gives_nothing() {
        let camera = Camera {
            width: 40,
            height: 40,
            focal: 40.0,
        };
        let flat = DepthImage::filled(40, 40, 1.0);
        let seen = Observed::new(flat, &camera, 0.2);
        let mut steep = DepthImage::filled(40, 40, f64::NAN);
        for (column, row) in steep.positions() {
            steep.set(
                column,
                row,
                1.0 + (crate::pixel::coordinate(column) - 20.0) * 0.1,
            );
        }
        let information = Information::of(&seen, &steep, &camera, 0.2);
        assert!(information.eigenvalues[5] < 1e-9);
    }
}
