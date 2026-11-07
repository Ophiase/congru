use std::sync::OnceLock;

use congru::Vector;
use nalgebra::Matrix3;

use crate::{Camera, DepthImage};

/// The image's normal at a pixel is a plane fitted this many pixels around
/// it: a depth noise of the size of the pixel spacing leaves nothing to read
/// between two neighbours.
const NORMAL_RADIUS: usize = 3;
/// Returns farther from a pixel's depth than this, in tolerances, belong to
/// another surface.
const NORMAL_REACH: f64 = 3.0;
/// A fit on fewer returns than this says nothing.
const LEAST_RETURNS: usize = 12;

/// A depth image with what a pose is judged against: the direction of its surface
/// at each pixel, fitted when first asked and kept. A pose only asks under its own
/// silhouette, a small share of the frame.
#[derive(Debug, Clone)]
pub struct Observed {
    pub depth: DepthImage,
    camera: Camera,
    normals: Vec<OnceLock<Option<Vector>>>,
    tolerance: f64,
}

impl PartialEq for Observed {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth && self.tolerance == other.tolerance
    }
}

impl Observed {
    /// `tolerance` must exceed the sensor's noise.
    #[must_use]
    pub fn new(depth: DepthImage, camera: &Camera, tolerance: f64) -> Self {
        let cells = depth.width() * depth.height();
        Self {
            depth,
            camera: *camera,
            normals: (0..cells).map(|_| OnceLock::new()).collect(),
            tolerance,
        }
    }

    /// How far from a surface a return still lies on it, as this was read at.
    #[must_use]
    pub const fn tolerance(&self) -> f64 {
        self.tolerance
    }

    pub(super) fn normal(&self, column: usize, row: usize) -> Option<Vector> {
        *self.normals[row * self.depth.width() + column]
            .get_or_init(|| fitted_normal(&self.depth, &self.camera, column, row, self.tolerance))
    }
}

/// The normal of the plane through the returns around a pixel.
fn fitted_normal(
    depth: &DepthImage,
    camera: &Camera,
    column: usize,
    row: usize,
    tolerance: f64,
) -> Option<Vector> {
    let centre = depth.get(column, row);
    if !centre.is_finite() {
        return None;
    }
    let columns =
        column.saturating_sub(NORMAL_RADIUS)..=(column + NORMAL_RADIUS).min(depth.width() - 1);
    let mut points = Vec::new();
    for across in columns {
        for down in
            row.saturating_sub(NORMAL_RADIUS)..=(row + NORMAL_RADIUS).min(depth.height() - 1)
        {
            let value = depth.get(across, down);
            if (value - centre).abs() <= NORMAL_REACH * tolerance {
                points.push(camera.back_project(across, down, value).coords);
            }
        }
    }
    if points.len() < LEAST_RETURNS {
        return None;
    }
    let mean = points.iter().sum::<Vector>() / crate::pixel::coordinate(points.len());
    let scatter = points.iter().fold(Matrix3::zeros(), |sum, point| {
        let offset = point - mean;
        sum + offset * offset.transpose()
    });
    let eigen = scatter.symmetric_eigen();
    Some(
        eigen
            .eigenvectors
            .column(eigen.eigenvalues.imin())
            .into_owned(),
    )
}
