use nalgebra::Unit;

use super::{Distance, OrientedCloud, PointCloud, Vector};
use crate::Error;

/// The plastic number, whose powers give the R2 low-discrepancy sequence.
const PLASTIC: f64 = 1.324_717_957_244_746;

/// A triangle mesh: shared vertices, triangles as triples of vertex indices.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleMesh {
    vertices: PointCloud,
    triangles: Box<[[u32; 3]]>,
}

impl TriangleMesh {
    /// # Errors
    /// [`Error::Empty`] without triangles, [`Error::DanglingTriangle`] for an
    /// index past the last vertex.
    pub fn new(vertices: PointCloud, triangles: impl Into<Vec<[u32; 3]>>) -> Result<Self, Error> {
        let triangles: Vec<[u32; 3]> = triangles.into();
        if triangles.is_empty() {
            return Err(Error::Empty { what: "mesh" });
        }
        let count = vertices.count();
        if let Some(triangle) = triangles
            .iter()
            .position(|corners| corners.iter().any(|&corner| corner as usize >= count))
        {
            return Err(Error::DanglingTriangle { triangle });
        }
        Ok(Self {
            vertices,
            triangles: triangles.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn vertices(&self) -> &PointCloud {
        &self.vertices
    }

    #[must_use]
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    /// Points spread evenly over the surface, about one per `spacing²` of
    /// area and at least one per triangle, each with its face normal.
    /// Deterministic: the same mesh always gives the same samples.
    ///
    /// # Errors
    /// [`Error::Degenerate`] when every triangle has zero area.
    pub fn sample_surface(&self, spacing: Distance) -> Result<OrientedCloud, Error> {
        let cell = spacing.get() * spacing.get();
        let mut points = Vec::new();
        let mut normals = Vec::new();
        for corners in &self.triangles {
            let [origin, first, second] =
                corners.map(|corner| self.vertices.points()[corner as usize]);
            let (along, across) = (first - origin, second - origin);
            let Some(normal) = Unit::try_new(along.cross(&across), f64::EPSILON) else {
                continue;
            };
            let area = along.cross(&across).norm() / 2.0;
            for (first_weight, second_weight) in r2_triangle(samples_for(area / cell)) {
                points.push(origin + along * first_weight + across * second_weight);
                normals.push(normal);
            }
        }
        if points.is_empty() {
            return Err(Error::Degenerate);
        }
        Ok(OrientedCloud::from_unit(
            PointCloud::new(points)?,
            normals.into_boxed_slice(),
        ))
    }
}

fn r2_triangle(count: usize) -> impl Iterator<Item = (f64, f64)> {
    let steps = Vector::new(1.0 / PLASTIC, 1.0 / (PLASTIC * PLASTIC), 0.0);
    (0..count).map(move |index| {
        let rank = super::count_as_float(index);
        let first = (0.5 + steps.x * rank).fract();
        let second = (0.5 + steps.y * rank).fract();
        if first + second > 1.0 {
            (1.0 - first, 1.0 - second)
        } else {
            (first, second)
        }
    })
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a non-negative sample count, clamped to at least one"
)]
fn samples_for(cells: f64) -> usize {
    cells.ceil().max(1.0) as usize
}
